use actix_web::web;

pub mod proxy;
mod routes_api;
mod stats_api;
mod token_api;
mod upstream_api;

/// Configure all admin endpoints under `/admin`.
pub fn configure_admin(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/upstreams")
            .route("", web::get().to(upstream_api::list))
            .route("", web::post().to(upstream_api::create))
            .route("/{id}", web::get().to(upstream_api::get))
            .route("/{id}", web::put().to(upstream_api::update))
            .route("/{id}", web::delete().to(upstream_api::delete)),
    )
    .service(
        web::scope("/api-tokens")
            .route("", web::get().to(token_api::list))
            .route("", web::post().to(token_api::create))
            .route("/{id}", web::get().to(token_api::get))
            .route("/{id}", web::put().to(token_api::update))
            .route("/{id}", web::delete().to(token_api::delete)),
    )
    .service(
        web::scope("/route-rules")
            .route("", web::get().to(routes_api::list))
            .route("", web::post().to(routes_api::create))
            .route("/{id}", web::get().to(routes_api::get))
            .route("/{id}", web::put().to(routes_api::update))
            .route("/{id}", web::delete().to(routes_api::delete)),
    )
    .service(
        web::scope("/health")
            .route("", web::get().to(stats_api::health_status))
            .route("/trigger", web::post().to(stats_api::trigger_health_check)),
    )
    .route("/stats/requests", web::get().to(stats_api::recent_requests));
}
