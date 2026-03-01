use actix_web::{web, App, HttpResponse, HttpServer};
use actix_files as fs;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tracing_actix_web::TracingLogger;

mod api;
mod config;
mod db;
mod error;
mod health;
mod middleware;
mod mock;
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

    tracing::info!(
        environment = ?config.environment,
        "Starting API-Transit in {:?} mode",
        config.environment
    );

    // Connect to database
    let db = db::init(&config.database_url).await?;

    // Run pending migrations
    db::run_migrations(&db).await?;
    tracing::info!("Database migrations completed");

    // Initialize mock data in development mode
    if config.environment.is_dev() {
        if let Err(e) = mock::init_mock_data(&db).await {
            tracing::warn!("Failed to initialize mock data: {}", e);
        }
    }

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

    // Get frontend dist path from env or use default
    let frontend_dist = std::env::var("FRONTEND_DIST")
        .unwrap_or_else(|_| "./frontend/dist".to_string());
    let frontend_path = std::path::Path::new(&frontend_dist);
    let serve_frontend = frontend_path.exists();

    if serve_frontend {
        tracing::info!(path = %frontend_dist, "Serving frontend static files");
    } else {
        tracing::warn!(path = %frontend_dist, "Frontend dist directory not found, skipping static file serving");
    }

    let frontend_dist_clone = frontend_dist.clone();

    HttpServer::new(move || {
        let mut app = App::new()
            .app_data(app_state.clone())
            .wrap(TracingLogger::default())
            // Public health probe
            .route(
                "/health",
                web::get()
                    .to(|| async { HttpResponse::Ok().json(serde_json::json!({"status": "ok"})) }),
            )
            // Public auth endpoints (no authentication required)
            .configure(api::configure_auth)
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
            );

        // Serve frontend static files if available
        if serve_frontend {
            let dist_path = frontend_dist_clone.clone();
            app = app
                .service(fs::Files::new("/", &frontend_dist_clone)
                    .index_file("index.html")
                    .use_last_modified(true)
                    .use_etag(true)
                    .prefer_utf8(true)
                    // SPA fallback: serve index.html for all non-API routes
                    .default_handler(web::get().to(move || {
                        let path = dist_path.clone();
                        async move {
                            let index_path = std::path::Path::new(&path).join("index.html");
                            fs::NamedFile::open_async(index_path).await
                        }
                    })));
        }

        app
    })
    .bind(&bind_addr)?
    .run()
    .await?;

    Ok(())
}
