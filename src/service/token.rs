use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::db::entities::api_tokens::{self, Entity as ApiToken};
use crate::error::AppError;

/// Hash a raw token using SHA-256, returning a lowercase hex string.
fn hash_token(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    hex::encode(hasher.finalize())
}

/// Generate a cryptographically-random API token string.
pub fn generate_token() -> String {
    // Format: at_<uuid_v4_without_dashes>
    format!("at_{}", Uuid::new_v4().simple())
}

/// DTO for creating a new token.
#[derive(Debug, Deserialize)]
pub struct CreateTokenDto {
    pub label: String,
    pub expires_at: Option<chrono::DateTime<Utc>>,
    pub rpm_limit: Option<i32>,
    pub tpm_limit: Option<i32>,
}

/// DTO for updating an existing token (all fields optional).
#[derive(Debug, Deserialize)]
pub struct UpdateTokenDto {
    pub label: Option<String>,
    pub enabled: Option<bool>,
    pub expires_at: Option<chrono::DateTime<Utc>>,
    pub rpm_limit: Option<i32>,
    pub tpm_limit: Option<i32>,
}

/// Public view of a token (never exposes the raw token or hash).
#[derive(Debug, Serialize)]
pub struct TokenDto {
    pub id: i64,
    pub label: String,
    pub expires_at: Option<chrono::DateTime<Utc>>,
    pub enabled: bool,
    pub rpm_limit: Option<i32>,
    pub tpm_limit: Option<i32>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

impl From<api_tokens::Model> for TokenDto {
    fn from(m: api_tokens::Model) -> Self {
        Self {
            id: m.id,
            label: m.label,
            expires_at: m.expires_at,
            enabled: m.enabled,
            rpm_limit: m.rpm_limit,
            tpm_limit: m.tpm_limit,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

/// Validate a raw Bearer token against the database.
/// Returns the full token model on success.
pub async fn validate(
    db: &DatabaseConnection,
    raw_token: &str,
) -> Result<api_tokens::Model, AppError> {
    let hash = hash_token(raw_token);

    let token = ApiToken::find()
        .filter(api_tokens::Column::TokenHash.eq(&hash))
        .filter(api_tokens::Column::Enabled.eq(true))
        .one(db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if let Some(expires_at) = token.expires_at {
        if expires_at < Utc::now() {
            return Err(AppError::Unauthorized);
        }
    }

    Ok(token)
}

/// List all tokens ordered by creation time (newest first).
pub async fn list(db: &DatabaseConnection) -> Result<Vec<api_tokens::Model>, AppError> {
    ApiToken::find()
        .order_by_desc(api_tokens::Column::CreatedAt)
        .all(db)
        .await
        .map_err(AppError::Database)
}

/// Get a single token by id.
pub async fn get(db: &DatabaseConnection, id: i64) -> Result<api_tokens::Model, AppError> {
    ApiToken::find_by_id(id)
        .one(db)
        .await?
        .ok_or(AppError::TokenNotFound { id })
}

/// Create a new token. Returns (model, raw_token).
/// The raw token is only available at creation time.
pub async fn create(
    db: &DatabaseConnection,
    dto: CreateTokenDto,
) -> Result<(api_tokens::Model, String), AppError> {
    let raw_token = generate_token();
    let token_hash = hash_token(&raw_token);
    let now = Utc::now();

    let model = api_tokens::ActiveModel {
        token_hash: Set(token_hash),
        label: Set(dto.label),
        expires_at: Set(dto.expires_at),
        enabled: Set(true),
        rpm_limit: Set(dto.rpm_limit),
        tpm_limit: Set(dto.tpm_limit),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(AppError::Database)?;

    Ok((model, raw_token))
}

/// Update mutable fields of an existing token.
pub async fn update(
    db: &DatabaseConnection,
    id: i64,
    dto: UpdateTokenDto,
) -> Result<api_tokens::Model, AppError> {
    let existing = get(db, id).await?;
    let now = Utc::now();
    let mut model: api_tokens::ActiveModel = existing.into();

    if let Some(label) = dto.label {
        model.label = Set(label);
    }
    if let Some(enabled) = dto.enabled {
        model.enabled = Set(enabled);
    }
    if let Some(expires_at) = dto.expires_at {
        model.expires_at = Set(Some(expires_at));
    }
    if let Some(rpm) = dto.rpm_limit {
        model.rpm_limit = Set(Some(rpm));
    }
    if let Some(tpm) = dto.tpm_limit {
        model.tpm_limit = Set(Some(tpm));
    }
    model.updated_at = Set(now);

    model.update(db).await.map_err(AppError::Database)
}

/// Disable a token (soft delete).
pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<(), AppError> {
    let existing = get(db, id).await?;
    let now = Utc::now();
    let mut model: api_tokens::ActiveModel = existing.into();
    model.enabled = Set(false);
    model.updated_at = Set(now);
    model.update(db).await?;
    Ok(())
}
