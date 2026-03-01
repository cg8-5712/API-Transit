use actix_web::{
    body::{EitherBody, MessageBody},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, HttpMessage, HttpResponse,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use serde_json::json;
use std::rc::Rc;

use crate::service::token as token_svc;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Token authentication middleware (for /api/* scope)
// ---------------------------------------------------------------------------

/// Validates `Authorization: Bearer <token>` against the `api_tokens` table.
pub struct TokenAuth;

impl<S, B> Transform<S, ServiceRequest> for TokenAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = TokenAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TokenAuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct TokenAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for TokenAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = Rc::clone(&self.service);

        Box::pin(async move {
            let state = match req.app_data::<web::Data<AppState>>() {
                Some(s) => s.clone(),
                None => return early_error(req, 500, "INTERNAL_ERROR", "Missing app state"),
            };

            let raw_token = match extract_bearer(&req) {
                Some(t) => t,
                None => {
                    return early_error(req, 401, "UNAUTHORIZED", "Missing Authorization header")
                }
            };

            match token_svc::validate(&state.db, &raw_token).await {
                Ok(token) => {
                    // Make the token id available to handlers for logging.
                    req.extensions_mut().insert(token.id);
                    let res = srv.call(req).await?.map_into_left_body();
                    Ok(res)
                }
                Err(_) => early_error(req, 401, "UNAUTHORIZED", "Invalid or expired token"),
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Admin authentication middleware (for /admin/* scope)
// ---------------------------------------------------------------------------

/// Validates `Authorization: Bearer <ADMIN_SECRET>` against the configured secret.
pub struct AdminAuth;

impl<S, B> Transform<S, ServiceRequest> for AdminAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AdminAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AdminAuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AdminAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AdminAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = Rc::clone(&self.service);

        Box::pin(async move {
            let state = match req.app_data::<web::Data<AppState>>() {
                Some(s) => s.clone(),
                None => return early_error(req, 500, "INTERNAL_ERROR", "Missing app state"),
            };

            let provided = match extract_bearer(&req) {
                Some(t) => t,
                None => {
                    return early_error(req, 401, "UNAUTHORIZED", "Missing Authorization header")
                }
            };

            if provided != state.config.admin_secret {
                return early_error(req, 401, "UNAUTHORIZED", "Invalid admin secret");
            }

            let res = srv.call(req).await?.map_into_left_body();
            Ok(res)
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn extract_bearer(req: &ServiceRequest) -> Option<String> {
    req.headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|t| t.to_string())
}

fn early_error<B>(
    req: ServiceRequest,
    status: u16,
    code: &'static str,
    message: &str,
) -> Result<ServiceResponse<EitherBody<B>>, Error>
where
    B: MessageBody + 'static,
{
    let (req, _payload) = req.into_parts();
    let status_code = actix_web::http::StatusCode::from_u16(status)
        .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
    let body = json!({"success": false, "error": {"code": code, "message": message}});
    let resp = HttpResponse::build(status_code)
        .json(body)
        .map_into_right_body();
    Ok(ServiceResponse::new(req, resp))
}
