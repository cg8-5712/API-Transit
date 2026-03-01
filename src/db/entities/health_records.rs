use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Result of a single upstream health probe.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "health_records")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub upstream_id: i64,
    pub checked_at: DateTimeUtc,
    /// Whether the probe succeeded.
    pub success: bool,
    /// Round-trip latency in milliseconds (`None` on failure).
    pub latency_ms: Option<i64>,
    /// Error description when `success = false`.
    pub error_message: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::upstreams::Entity",
        from = "Column::UpstreamId",
        to = "super::upstreams::Column::Id"
    )]
    Upstream,
}

impl Related<super::upstreams::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Upstream.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
