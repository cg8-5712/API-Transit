use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Path rewrite rule: maps an inbound path to an outbound path.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "route_rules")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    /// Human-readable rule name.
    pub name: String,
    /// Inbound path pattern.
    pub inbound_path: String,
    /// Outbound path to forward to the upstream.
    pub outbound_path: String,
    /// Matching strategy: `exact`, `prefix`, or `regex`.
    pub match_type: String,
    /// Pin to a specific upstream (`None` = use global load balancer).
    pub upstream_id: Option<i64>,
    /// Higher priority = evaluated first.
    pub priority: i32,
    /// Additional HTTP headers to inject (JSON object).
    pub extra_headers: Option<String>,
    /// Additional query parameters to inject (JSON object).
    pub extra_query: Option<String>,
    /// Whether this rule is active.
    pub enabled: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
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
