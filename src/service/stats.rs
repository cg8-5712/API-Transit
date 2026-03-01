use chrono::{DateTime, Duration, Utc};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::db::entities::{api_tokens, request_logs, upstreams};
use crate::error::AppError;

// ---------------------------------------------------------------------------
// Time period helpers
// ---------------------------------------------------------------------------

/// Named time windows supported by stats queries.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Period {
    #[serde(alias = "1d")]
    Day,
    #[serde(alias = "7d")]
    Week,
    #[serde(alias = "30d")]
    Month,
    All,
}

impl Period {
    pub fn since(self) -> Option<DateTime<Utc>> {
        match self {
            Self::Day => Some(Utc::now() - Duration::days(1)),
            Self::Week => Some(Utc::now() - Duration::days(7)),
            Self::Month => Some(Utc::now() - Duration::days(30)),
            Self::All => None,
        }
    }
}

impl Default for Period {
    fn default() -> Self {
        Self::Week
    }
}

// ---------------------------------------------------------------------------
// Stats DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct UpstreamStatDto {
    pub upstream_id: Option<i64>,
    pub upstream_name: Option<String>,
    pub total_requests: i64,
    pub success_requests: i64,
    pub error_requests: i64,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub total_request_bytes: i64,
    pub total_response_bytes: i64,
}

#[derive(Debug, Serialize)]
pub struct TokenStatDto {
    pub token_id: Option<i64>,
    pub token_label: Option<String>,
    pub total_requests: i64,
    pub success_requests: i64,
    pub error_requests: i64,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub total_request_bytes: i64,
}

#[derive(Debug, Serialize)]
pub struct DashboardSummary {
    pub period: String,
    pub total_requests: i64,
    pub success_requests: i64,
    pub error_requests: i64,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub total_upstreams: i64,
    pub healthy_upstreams: i64,
    pub active_tokens: i64,
    pub requests_per_day: Vec<DailyBucket>,
}

#[derive(Debug, Serialize)]
pub struct DailyBucket {
    pub date: String,
    pub total: i64,
    pub success: i64,
}

// ---------------------------------------------------------------------------
// Request log recording
// ---------------------------------------------------------------------------

/// Persist a single proxy request to the audit log (fire-and-forget).
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
    use sea_orm::{ActiveModelTrait, Set};
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

// ---------------------------------------------------------------------------
// Stats queries (in-process aggregation for cross-db compatibility)
// ---------------------------------------------------------------------------

/// Aggregate per-upstream statistics for the given time period.
pub async fn upstream_stats(
    db: &DatabaseConnection,
    period: Period,
) -> Result<Vec<UpstreamStatDto>, AppError> {
    let logs = fetch_logs(db, period).await?;

    // Collect all upstream names for display.
    let upstream_names = fetch_upstream_names(db).await?;

    // Aggregate.
    let mut map: HashMap<Option<i64>, Vec<&request_logs::Model>> = HashMap::new();
    for log in &logs {
        map.entry(log.upstream_id).or_default().push(log);
    }

    let mut results: Vec<UpstreamStatDto> = map
        .into_iter()
        .map(|(upstream_id, entries)| {
            let total = entries.len() as i64;
            let success = entries.iter().filter(|l| l.status_code < 400).count() as i64;
            let mut latencies: Vec<i64> = entries.iter().map(|l| l.latency_ms).collect();
            latencies.sort_unstable();
            let avg_lat = latencies.iter().sum::<i64>() as f64 / total as f64;
            let p95 = percentile(&latencies, 95);
            let req_bytes: i64 = entries.iter().map(|l| l.request_size).sum();
            let resp_bytes: i64 = entries.iter().map(|l| l.response_size).sum();

            UpstreamStatDto {
                upstream_id,
                upstream_name: upstream_id.and_then(|id| upstream_names.get(&id).cloned()),
                total_requests: total,
                success_requests: success,
                error_requests: total - success,
                success_rate: if total > 0 {
                    success as f64 / total as f64 * 100.0
                } else {
                    0.0
                },
                avg_latency_ms: avg_lat,
                p95_latency_ms: p95 as f64,
                total_request_bytes: req_bytes,
                total_response_bytes: resp_bytes,
            }
        })
        .collect();

    results.sort_by(|a, b| b.total_requests.cmp(&a.total_requests));
    Ok(results)
}

/// Aggregate per-token statistics for the given time period.
pub async fn token_stats(
    db: &DatabaseConnection,
    period: Period,
) -> Result<Vec<TokenStatDto>, AppError> {
    let logs = fetch_logs(db, period).await?;
    let token_labels = fetch_token_labels(db).await?;

    let mut map: HashMap<Option<i64>, Vec<&request_logs::Model>> = HashMap::new();
    for log in &logs {
        map.entry(log.token_id).or_default().push(log);
    }

    let mut results: Vec<TokenStatDto> = map
        .into_iter()
        .map(|(token_id, entries)| {
            let total = entries.len() as i64;
            let success = entries.iter().filter(|l| l.status_code < 400).count() as i64;
            let avg_lat = entries.iter().map(|l| l.latency_ms).sum::<i64>() as f64
                / total.max(1) as f64;
            let req_bytes: i64 = entries.iter().map(|l| l.request_size).sum();

            TokenStatDto {
                token_id,
                token_label: token_id.and_then(|id| token_labels.get(&id).cloned()),
                total_requests: total,
                success_requests: success,
                error_requests: total - success,
                success_rate: if total > 0 {
                    success as f64 / total as f64 * 100.0
                } else {
                    0.0
                },
                avg_latency_ms: avg_lat,
                total_request_bytes: req_bytes,
            }
        })
        .collect();

    results.sort_by(|a, b| b.total_requests.cmp(&a.total_requests));
    Ok(results)
}

/// Compute dashboard summary stats.
pub async fn dashboard_summary(
    db: &DatabaseConnection,
    period: Period,
) -> Result<DashboardSummary, AppError> {
    let logs = fetch_logs(db, period).await?;
    let total = logs.len() as i64;
    let success = logs.iter().filter(|l| l.status_code < 400).count() as i64;
    let avg_lat = if total > 0 {
        logs.iter().map(|l| l.latency_ms).sum::<i64>() as f64 / total as f64
    } else {
        0.0
    };

    // Upstream health counts.
    let all_upstreams = upstreams::Entity::find()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    let total_upstreams = all_upstreams.len() as i64;
    let healthy_upstreams = all_upstreams
        .iter()
        .filter(|u| u.enabled && u.is_healthy)
        .count() as i64;

    // Active token count.
    let active_tokens = api_tokens::Entity::find()
        .filter(api_tokens::Column::Enabled.eq(true))
        .count(db)
        .await
        .map_err(AppError::Database)? as i64;

    // Daily buckets (last 30 days max, or within period).
    let requests_per_day = build_daily_buckets(&logs, period);

    let period_label = match period {
        Period::Day => "1d",
        Period::Week => "7d",
        Period::Month => "30d",
        Period::All => "all",
    };

    Ok(DashboardSummary {
        period: period_label.to_string(),
        total_requests: total,
        success_requests: success,
        error_requests: total - success,
        success_rate: if total > 0 {
            success as f64 / total as f64 * 100.0
        } else {
            0.0
        },
        avg_latency_ms: avg_lat,
        total_upstreams,
        healthy_upstreams,
        active_tokens,
        requests_per_day,
    })
}

/// Return most recent N request logs.
pub async fn recent_logs(
    db: &DatabaseConnection,
    limit: u64,
) -> Result<Vec<request_logs::Model>, AppError> {
    request_logs::Entity::find()
        .order_by_desc(request_logs::Column::CreatedAt)
        .limit(limit)
        .all(db)
        .await
        .map_err(AppError::Database)
}

/// Return health check records for a single upstream (most recent first).
pub async fn health_history(
    db: &DatabaseConnection,
    upstream_id: i64,
    limit: u64,
) -> Result<Vec<crate::db::entities::health_records::Model>, AppError> {
    use crate::db::entities::health_records;
    health_records::Entity::find()
        .filter(health_records::Column::UpstreamId.eq(upstream_id))
        .order_by_desc(health_records::Column::CheckedAt)
        .limit(limit)
        .all(db)
        .await
        .map_err(AppError::Database)
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

async fn fetch_logs(
    db: &DatabaseConnection,
    period: Period,
) -> Result<Vec<request_logs::Model>, AppError> {
    let query = if let Some(since) = period.since() {
        request_logs::Entity::find()
            .filter(request_logs::Column::CreatedAt.gte(since))
            .order_by_asc(request_logs::Column::CreatedAt)
            .all(db)
            .await
    } else {
        request_logs::Entity::find()
            .order_by_asc(request_logs::Column::CreatedAt)
            .all(db)
            .await
    };
    query.map_err(AppError::Database)
}

async fn fetch_upstream_names(
    db: &DatabaseConnection,
) -> Result<HashMap<i64, String>, AppError> {
    let rows = upstreams::Entity::find()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    Ok(rows.into_iter().map(|u| (u.id, u.name)).collect())
}

async fn fetch_token_labels(
    db: &DatabaseConnection,
) -> Result<HashMap<i64, String>, AppError> {
    let rows = api_tokens::Entity::find()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    Ok(rows.into_iter().map(|t| (t.id, t.label)).collect())
}

/// Compute the Nth percentile of a sorted slice.
fn percentile(sorted: &[i64], p: u8) -> i64 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = (sorted.len() as f64 * p as f64 / 100.0).ceil() as usize;
    sorted[(idx.min(sorted.len()) - 1).max(0)]
}

/// Group request logs by calendar day (UTC) and count totals.
fn build_daily_buckets(logs: &[request_logs::Model], period: Period) -> Vec<DailyBucket> {
    let days = match period {
        Period::Day => 1,
        Period::Week => 7,
        Period::Month => 30,
        Period::All => 30, // cap at 30 days for "all" to keep response small
    };

    let now = Utc::now().date_naive();
    let mut buckets: Vec<DailyBucket> = (0..days)
        .rev()
        .map(|d| {
            let date = now - Duration::days(d as i64);
            DailyBucket {
                date: date.format("%Y-%m-%d").to_string(),
                total: 0,
                success: 0,
            }
        })
        .collect();

    for log in logs {
        let log_date = log.created_at.date_naive().format("%Y-%m-%d").to_string();
        if let Some(bucket) = buckets.iter_mut().find(|b| b.date == log_date) {
            bucket.total += 1;
            if log.status_code < 400 {
                bucket.success += 1;
            }
        }
    }

    buckets
}
