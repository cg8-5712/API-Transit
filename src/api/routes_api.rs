use actix_web::{web, HttpResponse, Result};
use serde_json::json;

use crate::error::AppError;
use crate::service::route_rule::{self, UpsertRouteRuleDto};
use crate::state::AppState;

/// `GET /admin/route-rules`
pub async fn list(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let models = route_rule::list(&state.db).await?;
    Ok(HttpResponse::Ok().json(json!({"success": true, "data": models})))
}

/// `GET /admin/route-rules/{id}`
pub async fn get(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let model = route_rule::get(&state.db, id).await?;
    Ok(HttpResponse::Ok().json(json!({"success": true, "data": model})))
}

/// `POST /admin/route-rules`
pub async fn create(
    state: web::Data<AppState>,
    body: web::Json<UpsertRouteRuleDto>,
) -> Result<HttpResponse, AppError> {
    let model = route_rule::create(&state.db, body.into_inner()).await?;
    Ok(HttpResponse::Created().json(json!({"success": true, "data": model})))
}

/// `PUT /admin/route-rules/{id}`
pub async fn update(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    body: web::Json<UpsertRouteRuleDto>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let model = route_rule::update(&state.db, id, body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(json!({"success": true, "data": model})))
}

/// `DELETE /admin/route-rules/{id}`
pub async fn delete(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    route_rule::delete(&state.db, id).await?;
    Ok(HttpResponse::Ok().json(json!({"success": true, "data": null})))
}
