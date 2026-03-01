use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// API token issued to downstream clients.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "api_tokens")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    /// SHA-256 hash of the raw token (never stored in plaintext).
    #[sea_orm(unique)]
    pub token_hash: String,
    /// Human-readable label for identification.
    pub label: String,
    /// Optional expiry; `None` means the token never expires.
    pub expires_at: Option<DateTimeUtc>,
    /// Whether the token is active.
    pub enabled: bool,
    /// Max requests per minute (`None` = unlimited).
    pub rpm_limit: Option<i32>,
    /// Max tokens per minute (`None` = unlimited).
    pub tpm_limit: Option<i32>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::request_logs::Entity")]
    RequestLogs,
}

impl Related<super::request_logs::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RequestLogs.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
