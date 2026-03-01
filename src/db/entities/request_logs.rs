use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Immutable audit log of every proxied request.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "request_logs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    /// Token that authenticated the request (`None` if unauthenticated somehow).
    pub token_id: Option<i64>,
    /// Upstream the request was forwarded to.
    pub upstream_id: Option<i64>,
    /// Original inbound path.
    pub path: String,
    /// HTTP method.
    pub method: String,
    /// Upstream response status code.
    pub status_code: i32,
    /// End-to-end latency in milliseconds.
    pub latency_ms: i64,
    /// Request body size in bytes.
    pub request_size: i64,
    /// Response body size in bytes (0 for streamed responses).
    pub response_size: i64,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::api_tokens::Entity",
        from = "Column::TokenId",
        to = "super::api_tokens::Column::Id"
    )]
    ApiToken,
    #[sea_orm(
        belongs_to = "super::upstreams::Entity",
        from = "Column::UpstreamId",
        to = "super::upstreams::Column::Id"
    )]
    Upstream,
}

impl Related<super::api_tokens::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ApiToken.def()
    }
}

impl Related<super::upstreams::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Upstream.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
