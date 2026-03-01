use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};

use crate::db::entities::upstreams::{self, Entity as Upstream};
use crate::error::AppError;
use crate::state::AppState;

/// DTO for creating or updating an upstream.
#[derive(Debug, Deserialize)]
pub struct UpsertUpstreamDto {
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub extra_headers: Option<String>,
    pub timeout_secs: Option<i32>,
    pub weight: Option<i32>,
    pub priority: Option<i32>,
    pub lb_strategy: Option<String>,
    pub enabled: Option<bool>,
}

/// Response DTO (hides the raw api_key).
#[derive(Debug, Serialize)]
pub struct UpstreamDto {
    pub id: i64,
    pub name: String,
    pub base_url: String,
    pub timeout_secs: i32,
    pub weight: i32,
    pub priority: i32,
    pub lb_strategy: String,
    pub enabled: bool,
    pub is_healthy: bool,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

impl From<upstreams::Model> for UpstreamDto {
    fn from(m: upstreams::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            base_url: m.base_url,
            timeout_secs: m.timeout_secs,
            weight: m.weight,
            priority: m.priority,
            lb_strategy: m.lb_strategy,
            enabled: m.enabled,
            is_healthy: m.is_healthy,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

/// List all upstreams ordered by priority then name.
pub async fn list(db: &DatabaseConnection) -> Result<Vec<upstreams::Model>, AppError> {
    Upstream::find()
        .order_by_asc(upstreams::Column::Priority)
        .order_by_asc(upstreams::Column::Name)
        .all(db)
        .await
        .map_err(AppError::Database)
}

/// Get a single upstream by id.
pub async fn get(db: &DatabaseConnection, id: i64) -> Result<upstreams::Model, AppError> {
    Upstream::find_by_id(id)
        .one(db)
        .await?
        .ok_or(AppError::UpstreamNotFound { id })
}

/// Create a new upstream.
pub async fn create(
    db: &DatabaseConnection,
    dto: UpsertUpstreamDto,
) -> Result<upstreams::Model, AppError> {
    let now = Utc::now();
    let model = upstreams::ActiveModel {
        name: Set(dto.name),
        base_url: Set(dto.base_url),
        api_key: Set(dto.api_key),
        extra_headers: Set(dto.extra_headers),
        timeout_secs: Set(dto.timeout_secs.unwrap_or(30)),
        weight: Set(dto.weight.unwrap_or(1)),
        priority: Set(dto.priority.unwrap_or(0)),
        lb_strategy: Set(dto.lb_strategy.unwrap_or_else(|| "round_robin".to_string())),
        enabled: Set(dto.enabled.unwrap_or(true)),
        is_healthy: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    model.insert(db).await.map_err(AppError::Database)
}

/// Update an existing upstream.
pub async fn update(
    db: &DatabaseConnection,
    id: i64,
    dto: UpsertUpstreamDto,
) -> Result<upstreams::Model, AppError> {
    let existing = get(db, id).await?;
    let now = Utc::now();
    let mut model: upstreams::ActiveModel = existing.into();
    model.name = Set(dto.name);
    model.base_url = Set(dto.base_url);
    model.api_key = Set(dto.api_key);
    model.extra_headers = Set(dto.extra_headers);
    model.timeout_secs = Set(dto.timeout_secs.unwrap_or(30));
    model.weight = Set(dto.weight.unwrap_or(1));
    model.priority = Set(dto.priority.unwrap_or(0));
    model.lb_strategy = Set(dto.lb_strategy.unwrap_or_else(|| "round_robin".to_string()));
    if let Some(enabled) = dto.enabled {
        model.enabled = Set(enabled);
    }
    model.updated_at = Set(now);
    model.update(db).await.map_err(AppError::Database)
}

/// Soft-delete: disable the upstream instead of removing it.
pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<(), AppError> {
    let existing = get(db, id).await?;
    let now = Utc::now();
    let mut model: upstreams::ActiveModel = existing.into();
    model.enabled = Set(false);
    model.updated_at = Set(now);
    model.update(db).await?;
    Ok(())
}

/// List all enabled and healthy upstreams for routing.
pub async fn get_available(db: &DatabaseConnection) -> Result<Vec<upstreams::Model>, AppError> {
    Upstream::find()
        .filter(upstreams::Column::Enabled.eq(true))
        .filter(upstreams::Column::IsHealthy.eq(true))
        .order_by_asc(upstreams::Column::Priority)
        .all(db)
        .await
        .map_err(AppError::Database)
}

/// List all upstreams (including disabled) for the health checker.
pub async fn get_all(db: &DatabaseConnection) -> Result<Vec<upstreams::Model>, AppError> {
    Upstream::find().all(db).await.map_err(AppError::Database)
}

/// Select an upstream using the configured load-balancing strategy.
pub async fn select(
    db: &DatabaseConnection,
    state: &AppState,
    preferred_id: Option<i64>,
) -> Result<upstreams::Model, AppError> {
    // A route rule may pin to a specific upstream.
    if let Some(id) = preferred_id {
        let up = get(db, id).await?;
        if up.enabled && up.is_healthy {
            return Ok(up);
        }
    }

    let upstreams = get_available(db).await?;
    if upstreams.is_empty() {
        return Err(AppError::NoAvailableUpstream);
    }

    // Determine the strategy from the first upstream's config (all active ones
    // are expected to share the same strategy in most deployments).
    let strategy = upstreams[0].lb_strategy.as_str();

    match strategy {
        "weighted" => select_weighted(&upstreams),
        "failover" => Ok(upstreams.into_iter().next().unwrap()),
        _ => {
            // round_robin (default)
            let idx = state
                .lb_counter
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                % upstreams.len();
            Ok(upstreams[idx].clone())
        }
    }
}

fn select_weighted(upstreams: &[upstreams::Model]) -> Result<upstreams::Model, AppError> {
    let total_weight: i32 = upstreams.iter().map(|u| u.weight.max(1)).sum();
    let mut rng_val = rand::random::<u32>() % (total_weight as u32);
    for up in upstreams {
        let w = up.weight.max(1) as u32;
        if rng_val < w {
            return Ok(up.clone());
        }
        rng_val -= w;
    }
    Ok(upstreams[0].clone())
}

/// Update the health flag on an upstream (called by the health scheduler).
pub async fn mark_health(db: &DatabaseConnection, id: i64, healthy: bool) -> Result<(), AppError> {
    let existing = get(db, id).await?;
    let now = Utc::now();
    let mut model: upstreams::ActiveModel = existing.into();
    model.is_healthy = Set(healthy);
    model.updated_at = Set(now);
    model.update(db).await?;
    Ok(())
}
