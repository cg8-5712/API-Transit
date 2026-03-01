use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Upstream service provider configuration.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "upstreams")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    /// Human-readable unique name.
    pub name: String,
    /// Base URL of the upstream (e.g. `https://api.openai.com`).
    pub base_url: String,
    /// API key injected as `Authorization: Bearer` when forwarding.
    pub api_key: Option<String>,
    /// Additional HTTP headers to inject (JSON object).
    pub extra_headers: Option<String>,
    /// Per-request timeout in seconds.
    pub timeout_secs: i32,
    /// Weight for weighted load balancing.
    pub weight: i32,
    /// Priority for failover strategy (lower = higher priority).
    pub priority: i32,
    /// Load balancing strategy: `round_robin`, `weighted`, `failover`.
    pub lb_strategy: String,
    /// Whether the upstream is administratively enabled.
    pub enabled: bool,
    /// Whether the upstream passed the last health check.
    pub is_healthy: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::route_rules::Entity")]
    RouteRules,
    #[sea_orm(has_many = "super::request_logs::Entity")]
    RequestLogs,
    #[sea_orm(has_many = "super::health_records::Entity")]
    HealthRecords,
}

impl Related<super::route_rules::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RouteRules.def()
    }
}

impl Related<super::request_logs::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RequestLogs.def()
    }
}

impl Related<super::health_records::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::HealthRecords.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
