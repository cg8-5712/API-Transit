use anyhow::Context;

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// HTTP server bind port.
    pub server_port: u16,
    /// Database connection URL (sqlite:// / postgresql:// / mysql://).
    pub database_url: String,
    /// Secret token for admin API access.
    pub admin_secret: String,
    /// Health check interval in seconds.
    pub health_check_interval_secs: u64,
}

impl AppConfig {
    /// Load configuration from environment variables.
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse::<u16>()
                .context("SERVER_PORT must be a valid port number")?,

            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://data/api_transit.db".to_string()),

            admin_secret: std::env::var("ADMIN_SECRET")
                .context("ADMIN_SECRET environment variable must be set")?,

            health_check_interval_secs: std::env::var("HEALTH_CHECK_INTERVAL")
                .unwrap_or_else(|_| "1800".to_string())
                .parse::<u64>()
                .context("HEALTH_CHECK_INTERVAL must be a valid number of seconds")?,
        })
    }
}
