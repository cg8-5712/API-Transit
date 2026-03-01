use sea_orm::DatabaseConnection;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use crate::config::AppConfig;

/// Shared application state injected into all Actix-web handlers via `web::Data<AppState>`.
pub struct AppState {
    /// Database connection pool.
    pub db: DatabaseConnection,
    /// Application configuration.
    pub config: AppConfig,
    /// Reusable HTTP client for proxying requests to upstreams.
    pub http_client: reqwest::Client,
    /// Round-robin counter for upstream load balancing.
    pub lb_counter: Arc<AtomicUsize>,
}
