use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
}

/// `POST /auth/login`
/// Admin login endpoint - validates password and returns admin token
pub async fn login(
    state: web::Data<AppState>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    // Validate password against ADMIN_SECRET
    if req.password != state.config.admin_secret {
        return Err(AppError::Unauthorized);
    }

    // Return the admin secret as token (in production, you might want to generate a JWT)
    let response = LoginResponse {
        token: state.config.admin_secret.clone(),
    };

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": response
    })))
}
