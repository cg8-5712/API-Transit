use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};

use crate::db::entities::route_rules::{self, Entity as RouteRule};
use crate::error::AppError;

/// DTO for creating or updating a route rule.
#[derive(Debug, Deserialize)]
pub struct UpsertRouteRuleDto {
    pub name: String,
    pub inbound_path: String,
    pub outbound_path: String,
    pub match_type: Option<String>,
    pub upstream_id: Option<i64>,
    pub priority: Option<i32>,
    pub extra_headers: Option<String>,
    pub extra_query: Option<String>,
    pub enabled: Option<bool>,
}

/// List all route rules ordered by priority descending (higher first).
pub async fn list(db: &DatabaseConnection) -> Result<Vec<route_rules::Model>, AppError> {
    RouteRule::find()
        .order_by_desc(route_rules::Column::Priority)
        .order_by_asc(route_rules::Column::Id)
        .all(db)
        .await
        .map_err(AppError::Database)
}

/// Get enabled rules only — used by the proxy at request time.
pub async fn list_enabled(db: &DatabaseConnection) -> Result<Vec<route_rules::Model>, AppError> {
    RouteRule::find()
        .filter(route_rules::Column::Enabled.eq(true))
        .order_by_desc(route_rules::Column::Priority)
        .all(db)
        .await
        .map_err(AppError::Database)
}

/// Get a single route rule by id.
pub async fn get(db: &DatabaseConnection, id: i64) -> Result<route_rules::Model, AppError> {
    RouteRule::find_by_id(id)
        .one(db)
        .await?
        .ok_or(AppError::RouteRuleNotFound { id })
}

/// Create a new route rule.
pub async fn create(
    db: &DatabaseConnection,
    dto: UpsertRouteRuleDto,
) -> Result<route_rules::Model, AppError> {
    let now = Utc::now();
    let model = route_rules::ActiveModel {
        name: Set(dto.name),
        inbound_path: Set(dto.inbound_path),
        outbound_path: Set(dto.outbound_path),
        match_type: Set(dto.match_type.unwrap_or_else(|| "exact".to_string())),
        upstream_id: Set(dto.upstream_id),
        priority: Set(dto.priority.unwrap_or(0)),
        extra_headers: Set(dto.extra_headers),
        extra_query: Set(dto.extra_query),
        enabled: Set(dto.enabled.unwrap_or(true)),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    model.insert(db).await.map_err(AppError::Database)
}

/// Update an existing route rule.
pub async fn update(
    db: &DatabaseConnection,
    id: i64,
    dto: UpsertRouteRuleDto,
) -> Result<route_rules::Model, AppError> {
    let existing = get(db, id).await?;
    let now = Utc::now();
    let mut model: route_rules::ActiveModel = existing.into();
    model.name = Set(dto.name);
    model.inbound_path = Set(dto.inbound_path);
    model.outbound_path = Set(dto.outbound_path);
    model.match_type = Set(dto.match_type.unwrap_or_else(|| "exact".to_string()));
    model.upstream_id = Set(dto.upstream_id);
    model.priority = Set(dto.priority.unwrap_or(0));
    model.extra_headers = Set(dto.extra_headers);
    model.extra_query = Set(dto.extra_query);
    if let Some(enabled) = dto.enabled {
        model.enabled = Set(enabled);
    }
    model.updated_at = Set(now);
    model.update(db).await.map_err(AppError::Database)
}

/// Delete a route rule by id.
pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<(), AppError> {
    let existing = get(db, id).await?;
    let model: route_rules::ActiveModel = existing.into();
    model.delete(db).await?;
    Ok(())
}
