use std::time::Duration;

use actix_web::web;
use chrono::Utc;
use sea_orm::ActiveModelTrait;
use sea_orm::Set;

use crate::db::entities::{health_records, upstreams};
use crate::state::AppState;

/// Run the health check loop indefinitely.
/// Spawned as a background Tokio task in `main`.
pub async fn run(state: web::Data<AppState>) {
    let interval_secs = state.config.health_check_interval_secs;
    tracing::info!(
        interval_secs = interval_secs,
        "Health check scheduler started"
    );

    let mut ticker = tokio::time::interval(Duration::from_secs(interval_secs));
    // Tick immediately once at startup, then repeat.
    loop {
        ticker.tick().await;
        check_all(&state).await;
    }
}

/// Check all enabled upstreams once and update their health status.
pub async fn check_all(state: &AppState) {
    let upstreams = match crate::service::upstream::get_all(&state.db).await {
        Ok(list) => list,
        Err(e) => {
            tracing::error!(error = %e, "Health check: failed to fetch upstreams");
            return;
        }
    };

    for upstream in upstreams {
        if !upstream.enabled {
            continue;
        }
        check_one(state, &upstream).await;
    }
}

async fn check_one(state: &AppState, upstream: &upstreams::Model) {
    let start = std::time::Instant::now();
    let result = probe(&state.http_client, &upstream.base_url).await;
    let latency_ms = start.elapsed().as_millis() as i64;

    let (success, error_message) = match &result {
        Ok(_) => (true, None),
        Err(e) => (false, Some(e.to_string())),
    };

    // Persist the record.
    let record = health_records::ActiveModel {
        upstream_id: Set(upstream.id),
        checked_at: Set(Utc::now()),
        success: Set(success),
        latency_ms: Set(if success { Some(latency_ms) } else { None }),
        error_message: Set(error_message),
        ..Default::default()
    };
    if let Err(e) = record.insert(&state.db).await {
        tracing::error!(
            upstream_id = upstream.id,
            error = %e,
            "Health check: failed to save record"
        );
    }

    // Update the upstream's health flag.
    if let Err(e) = crate::service::upstream::mark_health(&state.db, upstream.id, success).await {
        tracing::error!(
            upstream_id = upstream.id,
            error = %e,
            "Health check: failed to update upstream health"
        );
    }

    tracing::info!(
        upstream_id = upstream.id,
        upstream_name = %upstream.name,
        success = success,
        latency_ms = latency_ms,
        "Health check completed"
    );
}

/// Send a lightweight GET probe to the upstream base URL.
/// 4xx responses are treated as "reachable" (the upstream is running);
/// only network errors and 5xx responses are treated as failures.
async fn probe(client: &reqwest::Client, base_url: &str) -> anyhow::Result<()> {
    let url = base_url.trim_end_matches('/');
    let response = client
        .get(url)
        .timeout(Duration::from_secs(10))
        .send()
        .await?;

    if response.status().is_server_error() {
        anyhow::bail!("upstream returned server error: {}", response.status());
    }

    Ok(())
}
