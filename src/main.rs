use actix_web::{web, App, HttpResponse, HttpServer};
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tracing_actix_web::TracingLogger;

mod api;
mod config;
mod db;
mod error;
mod health;
mod middleware;
mod proxy;
mod service;
mod state;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file (ignore if missing)
    dotenvy::dotenv().ok();

    // Initialize structured tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = config::AppConfig::from_env()?;

    // Ensure the data directory exists for SQLite
    if config.database_url.starts_with("sqlite://") {
        let db_path = config.database_url.trim_start_matches("sqlite://");
        if let Some(parent) = std::path::Path::new(db_path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
    }

    // Connect to database
    let db = db::init(&config.database_url).await?;

    // Run pending migrations
    db::run_migrations(&db).await?;
    tracing::info!("Database migrations completed");

    // Build shared HTTP client
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let app_state = web::Data::new(state::AppState {
        db: db.clone(),
        config: config.clone(),
        http_client,
        lb_counter: Arc::new(AtomicUsize::new(0)),
    });

    // Spawn health check scheduler as background task
    let scheduler_state = app_state.clone();
    tokio::spawn(async move {
        health::scheduler::run(scheduler_state).await;
    });

    let bind_addr = format!("0.0.0.0:{}", config.server_port);
    tracing::info!(addr = %bind_addr, "Starting API-Transit server");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(TracingLogger::default())
            // Public health probe
            .route(
                "/health",
                web::get()
                    .to(|| async { HttpResponse::Ok().json(serde_json::json!({"status": "ok"})) }),
            )
            // Admin management API (protected by ADMIN_SECRET)
            .service(
                web::scope("/admin")
                    .wrap(middleware::auth::AdminAuth)
                    .configure(api::configure_admin),
            )
            // Proxy entry point (protected by API token)
            .service(
                web::scope("/api")
                    .wrap(middleware::auth::TokenAuth)
                    .default_service(web::route().to(api::proxy::proxy_handler)),
            )
    })
    .bind(&bind_addr)?
    .run()
    .await?;

    Ok(())
}
