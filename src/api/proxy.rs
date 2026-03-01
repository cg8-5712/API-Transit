use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use bytes::BytesMut;
use futures_util::StreamExt;
use std::str::FromStr;

use crate::error::AppError;
use crate::proxy::{forward, rewrite};
use crate::service::{route_rule, stats, upstream};
use crate::state::AppState;

/// Core proxy handler — catches all requests under `/api/*`.
///
/// Pipeline:
/// 1. Read request body.
/// 2. Apply route rewriting (first matching enabled rule wins).
/// 3. Select an upstream via load balancing.
/// 4. Forward the request and stream the response back.
/// 5. Record the request log asynchronously.
pub async fn proxy_handler(
    req: HttpRequest,
    mut payload: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    // Collect request body (supports LLM API payloads up to a reasonable size).
    let mut body_buf = BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk =
            chunk.map_err(|e| AppError::Internal(anyhow::anyhow!("payload read error: {}", e)))?;
        body_buf.extend_from_slice(&chunk);
    }
    let body = body_buf.freeze();

    let inbound_path = req.uri().path().to_string();
    let query = req.query_string().to_string();

    // Token id was inserted by the TokenAuth middleware.
    let token_id = req.extensions().get::<i64>().copied();

    // --- Route rewriting ---
    let rules = route_rule::list_enabled(&state.db).await?;
    let (target_path, preferred_upstream_id, upstream_ids) = match rewrite::rewrite(&rules, &inbound_path) {
        Some(r) => (r.path, r.upstream_id, r.upstream_ids),
        None => (inbound_path.clone(), None, Vec::new()),
    };

    // --- Upstream selection ---
    // Priority: preferred_upstream_id > upstream_ids > all available
    let upstream_model = if let Some(id) = preferred_upstream_id {
        // Single upstream specified (legacy)
        upstream::get(&state.db, id).await?
    } else if !upstream_ids.is_empty() {
        // Multiple upstreams specified for load balancing
        upstream::select_from_list(&state.db, &state, &upstream_ids).await?
    } else {
        // Use all available upstreams
        upstream::select(&state.db, &state, None).await?
    };

    let start = std::time::Instant::now();

    // --- Convert actix method to reqwest method ---
    let method = reqwest::Method::from_str(req.method().as_str()).unwrap_or(reqwest::Method::GET);

    // --- Forward the request ---
    let upstream_resp = forward::forward(
        &state.http_client,
        &upstream_model,
        &target_path,
        &query,
        method,
        req.headers(),
        body.clone(),
    )
    .await?;

    let latency_ms = start.elapsed().as_millis() as i64;
    let status_code = upstream_resp.status().as_u16() as i32;

    // --- Build downstream response ---
    let resp_status = actix_web::http::StatusCode::from_u16(upstream_resp.status().as_u16())
        .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
    let resp_headers = upstream_resp.headers().clone();

    let mut builder = HttpResponse::build(resp_status);

    // Forward safe upstream response headers.
    const SKIP_RESP_HEADERS: &[&str] = &[
        "transfer-encoding",
        "connection",
        "keep-alive",
        "content-length",
    ];
    for (name, value) in resp_headers.iter() {
        let name_str = name.as_str().to_lowercase();
        if SKIP_RESP_HEADERS.contains(&name_str.as_str()) {
            continue;
        }
        if let Ok(actix_name) = actix_web::http::header::HeaderName::from_str(name.as_str()) {
            if let Ok(actix_val) =
                actix_web::http::header::HeaderValue::from_bytes(value.as_bytes())
            {
                builder.insert_header((actix_name, actix_val));
            }
        }
    }

    // --- Record request log (fire-and-forget) ---
    let request_size = body.len() as i64;
    let upstream_id = upstream_model.id;
    let path_clone = inbound_path.clone();
    let method_str = req.method().to_string();
    let db = state.db.clone();
    tokio::spawn(async move {
        if let Err(e) = stats::record_request(
            db,
            token_id,
            Some(upstream_id),
            path_clone,
            method_str,
            status_code,
            latency_ms,
            request_size,
            0, // response size unknown for streamed responses
        )
        .await
        {
            tracing::warn!(error = %e, "Failed to record request log");
        }
    });

    // --- Stream the upstream response back to the client ---
    let stream = upstream_resp.bytes_stream();
    Ok(builder.streaming(stream))
}
