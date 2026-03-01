use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde_json::json;

/// Unified application error type.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Upstream not found: id={id}")]
    UpstreamNotFound { id: i64 },

    #[error("Token not found: id={id}")]
    TokenNotFound { id: i64 },

    #[error("Route rule not found: id={id}")]
    RouteRuleNotFound { id: i64 },

    #[error("Authentication failed")]
    Unauthorized,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("No available upstream")]
    NoAvailableUpstream,

    #[error("Upstream unavailable: {0}")]
    UpstreamUnavailable(String),

    #[error("Route rewrite failed: {0}")]
    RewriteError(String),

    #[error("Proxy request failed: {0}")]
    ProxyError(String),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            Self::UpstreamNotFound { .. } => StatusCode::NOT_FOUND,
            Self::TokenNotFound { .. } => StatusCode::NOT_FOUND,
            Self::RouteRuleNotFound { .. } => StatusCode::NOT_FOUND,
            Self::NoAvailableUpstream => StatusCode::BAD_GATEWAY,
            Self::UpstreamUnavailable(_) => StatusCode::BAD_GATEWAY,
            Self::ProxyError(_) => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(json!({
            "success": false,
            "error": {
                "code": self.error_code(),
                "message": self.to_string()
            }
        }))
    }
}

impl AppError {
    fn error_code(&self) -> &'static str {
        match self {
            Self::Database(_) => "DATABASE_ERROR",
            Self::UpstreamNotFound { .. } => "UPSTREAM_NOT_FOUND",
            Self::TokenNotFound { .. } => "TOKEN_NOT_FOUND",
            Self::RouteRuleNotFound { .. } => "ROUTE_RULE_NOT_FOUND",
            Self::Unauthorized => "UNAUTHORIZED",
            Self::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
            Self::NoAvailableUpstream => "NO_AVAILABLE_UPSTREAM",
            Self::UpstreamUnavailable(_) => "UPSTREAM_UNAVAILABLE",
            Self::RewriteError(_) => "REWRITE_ERROR",
            Self::ProxyError(_) => "PROXY_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }
}
