use actix_web::{web, HttpResponse, Result};
use serde::Deserialize;
use serde_json::json;

use crate::error::AppError;
use crate::service::stats::{self, Period};
use crate::service::upstream;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct PeriodQuery {
    #[serde(default)]
    pub period: Period,
}

// ---------------------------------------------------------------------------
// Dashboard summary
// ---------------------------------------------------------------------------

/// `GET /admin/stats/summary?period=7d`
pub async fn summary(
    state: web::Data<AppState>,
    query: web::Query<PeriodQuery>,
) -> Result<HttpResponse, AppError> {
    let data = stats::dashboard_summary(&state.db, query.period).await?;
    Ok(HttpResponse::Ok().json(json!({"success": true, "data": data})))
}

// ---------------------------------------------------------------------------
// Upstream stats
// ---------------------------------------------------------------------------

/// `GET /admin/stats/upstream?period=7d`
pub async fn upstream_stats(
    state: web::Data<AppState>,
    query: web::Query<PeriodQuery>,
) -> Result<HttpResponse, AppError> {
    let data = stats::upstream_stats(&state.db, query.period).await?;
    Ok(HttpResponse::Ok().json(json!({"success": true, "data": data})))
}

// ---------------------------------------------------------------------------
// Token stats
// ---------------------------------------------------------------------------

/// `GET /admin/stats/token?period=7d`
pub async fn token_stats(
    state: web::Data<AppState>,
    query: web::Query<PeriodQuery>,
) -> Result<HttpResponse, AppError> {
    let data = stats::token_stats(&state.db, query.period).await?;
    Ok(HttpResponse::Ok().json(json!({"success": true, "data": data})))
}

// ---------------------------------------------------------------------------
// Request logs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    #[serde(default = "default_limit")]
    pub limit: u64,
}

fn default_limit() -> u64 {
    100
}

/// `GET /admin/stats/requests?limit=100`
pub async fn recent_requests(
    state: web::Data<AppState>,
    query: web::Query<LogsQuery>,
) -> Result<HttpResponse, AppError> {
    let limit = query.limit.min(1000);
    let data = stats::recent_logs(&state.db, limit).await?;
    Ok(HttpResponse::Ok().json(json!({"success": true, "data": data})))
}

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

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
                "updated_at": u.updated_at,
            })
        })
        .collect();
    Ok(HttpResponse::Ok().json(json!({"success": true, "data": statuses})))
}

/// `GET /admin/health/{upstream_id}/history?limit=50`
#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    #[serde(default = "default_history_limit")]
    pub limit: u64,
}

fn default_history_limit() -> u64 {
    50
}

pub async fn health_history(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    query: web::Query<HistoryQuery>,
) -> Result<HttpResponse, AppError> {
    let upstream_id = path.into_inner();
    let data = stats::health_history(&state.db, upstream_id, query.limit.min(200)).await?;
    Ok(HttpResponse::Ok().json(json!({"success": true, "data": data})))
}

/// `POST /admin/health/trigger`
/// Triggers an immediate health check for all upstreams.
pub async fn trigger_health_check(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let state_clone = state.clone();
    tokio::spawn(async move {
        crate::health::scheduler::check_all(&state_clone).await;
    });
    Ok(HttpResponse::Accepted()
        .json(json!({"success": true, "data": {"message": "Health check triggered"}})))
}
