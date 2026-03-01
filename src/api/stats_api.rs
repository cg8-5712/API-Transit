use actix_web::{web, HttpResponse, Result};
use serde_json::json;

use crate::error::AppError;
use crate::service::{stats, upstream};
use crate::state::AppState;

/// `GET /admin/health`
/// Returns current health status of all upstreams.
pub async fn health_status(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let models = upstream::get_all(&state.db).await?;
    let statuses: Vec<_> = models
        .into_iter()
        .map(|u| {
            json!({
                "id": u.id,
                "name": u.name,
                "base_url": u.base_url,
                "enabled": u.enabled,
                "is_healthy": u.is_healthy,
            })
        })
        .collect();
    Ok(HttpResponse::Ok().json(json!({"success": true, "data": statuses})))
}

/// `POST /admin/health/trigger`
/// Triggers an immediate health check for all upstreams.
pub async fn trigger_health_check(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    // Run checks in the background so the response returns immediately.
    let state_clone = state.clone();
    tokio::spawn(async move {
        crate::health::scheduler::check_all(&state_clone).await;
    });
    Ok(HttpResponse::Accepted()
        .json(json!({"success": true, "data": {"message": "Health check triggered"}})))
}

/// `GET /admin/stats/requests`
/// Returns the most recent 1000 request log entries.
pub async fn recent_requests(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let logs = stats::recent_logs(&state.db).await?;
    Ok(HttpResponse::Ok().json(json!({"success": true, "data": logs})))
}
