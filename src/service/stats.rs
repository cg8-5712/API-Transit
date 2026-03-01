use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, Set,
};
use serde::Serialize;

use crate::db::entities::request_logs;
use crate::error::AppError;

/// Aggregate statistics for a single upstream.
#[derive(Debug, Serialize)]
pub struct UpstreamStats {
    pub upstream_id: i64,
    pub total_requests: u64,
    pub success_requests: u64,
    pub avg_latency_ms: f64,
}

/// Record a completed proxy request in the audit log.
pub async fn record_request(
    db: DatabaseConnection,
    token_id: Option<i64>,
    upstream_id: Option<i64>,
    path: String,
    method: String,
    status_code: i32,
    latency_ms: i64,
    request_size: i64,
    response_size: i64,
) -> Result<(), AppError> {
    let model = request_logs::ActiveModel {
        token_id: Set(token_id),
        upstream_id: Set(upstream_id),
        path: Set(path),
        method: Set(method),
        status_code: Set(status_code),
        latency_ms: Set(latency_ms),
        request_size: Set(request_size),
        response_size: Set(response_size),
        created_at: Set(Utc::now()),
        ..Default::default()
    };
    model.insert(&db).await.map_err(AppError::Database)?;
    Ok(())
}

/// Fetch recent request logs (last 1000).
pub async fn recent_logs(db: &DatabaseConnection) -> Result<Vec<request_logs::Model>, AppError> {
    use sea_orm::QueryOrder;
    request_logs::Entity::find()
        .order_by_desc(request_logs::Column::CreatedAt)
        .limit(1000)
        .all(db)
        .await
        .map_err(AppError::Database)
}
