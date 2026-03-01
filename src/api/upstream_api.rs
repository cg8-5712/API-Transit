use actix_web::{web, HttpResponse, Result};
use serde_json::json;

use crate::error::AppError;
use crate::service::upstream::{self, UpsertUpstreamDto, UpstreamDto};
use crate::state::AppState;

/// `GET /admin/upstreams`
pub async fn list(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let models = upstream::list(&state.db).await?;
    let dtos: Vec<UpstreamDto> = models.into_iter().map(UpstreamDto::from).collect();
    Ok(HttpResponse::Ok().json(json!({"success": true, "data": dtos})))
}

/// `GET /admin/upstreams/{id}`
pub async fn get(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let model = upstream::get(&state.db, id).await?;
    Ok(HttpResponse::Ok().json(json!({"success": true, "data": UpstreamDto::from(model)})))
}

/// `POST /admin/upstreams`
pub async fn create(
    state: web::Data<AppState>,
    body: web::Json<UpsertUpstreamDto>,
) -> Result<HttpResponse, AppError> {
    let model = upstream::create(&state.db, body.into_inner()).await?;
    Ok(HttpResponse::Created().json(json!({"success": true, "data": UpstreamDto::from(model)})))
}

/// `PUT /admin/upstreams/{id}`
pub async fn update(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    body: web::Json<UpsertUpstreamDto>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let model = upstream::update(&state.db, id, body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(json!({"success": true, "data": UpstreamDto::from(model)})))
}

/// `DELETE /admin/upstreams/{id}`
pub async fn delete(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    upstream::delete(&state.db, id).await?;
    Ok(HttpResponse::Ok().json(json!({"success": true, "data": null})))
}
