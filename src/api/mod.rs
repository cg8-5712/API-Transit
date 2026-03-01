use actix_web::web;

pub mod proxy;
mod routes_api;
mod stats_api;
mod token_api;
mod upstream_api;
mod auth_api;

/// Configure all admin endpoints under `/admin`.
pub fn configure_admin(cfg: &mut web::ServiceConfig) {
    // Upstream CRUD
    cfg.service(
        web::scope("/upstreams")
            .route("", web::get().to(upstream_api::list))
            .route("", web::post().to(upstream_api::create))
            .route("/{id}", web::get().to(upstream_api::get))
            .route("/{id}", web::put().to(upstream_api::update))
            .route("/{id}", web::delete().to(upstream_api::delete)),
    )
    // API Token CRUD
    .service(
        web::scope("/api-tokens")
            .route("", web::get().to(token_api::list))
            .route("", web::post().to(token_api::create))
            .route("/{id}", web::get().to(token_api::get))
            .route("/{id}", web::put().to(token_api::update))
            .route("/{id}", web::delete().to(token_api::delete)),
    )
    // Route rules CRUD
    .service(
        web::scope("/route-rules")
            .route("", web::get().to(routes_api::list))
            .route("", web::post().to(routes_api::create))
            .route("/{id}", web::get().to(routes_api::get))
            .route("/{id}", web::put().to(routes_api::update))
            .route("/{id}", web::delete().to(routes_api::delete)),
    )
    // Health check status + history + manual trigger
    .service(
        web::scope("/health")
            .route("", web::get().to(stats_api::health_status))
            .route("/trigger", web::post().to(stats_api::trigger_health_check))
            .route("/{upstream_id}/history", web::get().to(stats_api::health_history)),
    )
    // Statistics (P2)
    .service(
        web::scope("/stats")
            .route("/summary", web::get().to(stats_api::summary))
            .route("/upstream", web::get().to(stats_api::upstream_stats))
            .route("/token", web::get().to(stats_api::token_stats))
            .route("/requests", web::get().to(stats_api::recent_requests)),
    );
}

/// Configure public auth endpoints (no authentication required)
pub fn configure_auth(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/login", web::post().to(auth_api::login)),
    );
}
