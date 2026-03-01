use actix_web::{web, HttpResponse, Result};
use serde_json::json;

use crate::error::AppError;
use crate::service::token::{self, CreateTokenDto, TokenDto, UpdateTokenDto};
use crate::state::AppState;

/// `GET /admin/api-tokens`
pub async fn list(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let models = token::list(&state.db).await?;
    let dtos: Vec<TokenDto> = models.into_iter().map(TokenDto::from).collect();
    Ok(HttpResponse::Ok().json(json!({"success": true, "data": dtos})))
}

/// `GET /admin/api-tokens/{id}`
pub async fn get(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let model = token::get(&state.db, id).await?;
    Ok(HttpResponse::Ok().json(json!({"success": true, "data": TokenDto::from(model)})))
}

/// `POST /admin/api-tokens`
/// Returns the raw token in the response — it will not be shown again.
pub async fn create(
    state: web::Data<AppState>,
    body: web::Json<CreateTokenDto>,
) -> Result<HttpResponse, AppError> {
    let (model, raw_token) = token::create(&state.db, body.into_inner()).await?;
    Ok(HttpResponse::Created().json(json!({
        "success": true,
        "data": {
            "token": raw_token,  // shown once only
            "id": model.id,
            "label": model.label,
            "expires_at": model.expires_at,
            "enabled": model.enabled,
            "created_at": model.created_at,
        }
    })))
}

/// `PUT /admin/api-tokens/{id}`
pub async fn update(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    body: web::Json<UpdateTokenDto>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let model = token::update(&state.db, id, body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(json!({"success": true, "data": TokenDto::from(model)})))
}

/// `DELETE /admin/api-tokens/{id}`
pub async fn delete(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    token::delete(&state.db, id).await?;
    Ok(HttpResponse::Ok().json(json!({"success": true, "data": null})))
}
