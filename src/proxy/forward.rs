use actix_web::http::header::HeaderMap;
use anyhow::Context;
use reqwest::header::{HeaderName, HeaderValue, AUTHORIZATION};
use std::str::FromStr;

use crate::db::entities::upstreams;
use crate::error::AppError;

/// Headers that must not be forwarded to the upstream (hop-by-hop headers).
const HOP_BY_HOP: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailers",
    "transfer-encoding",
    "upgrade",
    "host",
];

/// Forward an HTTP request to the given upstream and return the raw response.
pub async fn forward(
    client: &reqwest::Client,
    upstream: &upstreams::Model,
    path: &str,
    query: &str,
    method: reqwest::Method,
    incoming_headers: &HeaderMap,
    body: bytes::Bytes,
) -> Result<reqwest::Response, AppError> {
    let base = upstream.base_url.trim_end_matches('/');
    let target_url = if query.is_empty() {
        format!("{}{}", base, path)
    } else {
        format!("{}{}?{}", base, path, query)
    };

    let mut headers = reqwest::header::HeaderMap::new();

    // Copy safe incoming headers.
    for (name, value) in incoming_headers.iter() {
        let name_lower = name.as_str().to_lowercase();
        if HOP_BY_HOP.contains(&name_lower.as_str()) || name_lower == "authorization" {
            continue;
        }
        if let Ok(n) = HeaderName::from_str(name.as_str()) {
            if let Ok(v) = HeaderValue::from_bytes(value.as_bytes()) {
                headers.insert(n, v);
            }
        }
    }

    // Inject upstream API key.
    if let Some(ref api_key) = upstream.api_key {
        let value = HeaderValue::from_str(&format!("Bearer {}", api_key))
            .context("upstream api_key contains invalid header characters")
            .map_err(AppError::Internal)?;
        headers.insert(AUTHORIZATION, value);
    }

    // Inject extra headers configured on the upstream.
    if let Some(ref extra_json) = upstream.extra_headers {
        if let Ok(map) =
            serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(extra_json)
        {
            for (k, v) in &map {
                if let (Ok(name), Some(val_str)) = (HeaderName::from_str(k), v.as_str()) {
                    if let Ok(val) = HeaderValue::from_str(val_str) {
                        headers.insert(name, val);
                    }
                }
            }
        }
    }

    let timeout = std::time::Duration::from_secs(upstream.timeout_secs as u64);

    let response = client
        .request(method, &target_url)
        .headers(headers)
        .body(body)
        .timeout(timeout)
        .send()
        .await
        .map_err(|e| AppError::ProxyError(e.to_string()))?;

    Ok(response)
}
