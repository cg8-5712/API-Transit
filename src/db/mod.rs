use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;

pub mod entities;
pub mod migrations;

/// Initialize the database connection pool.
pub async fn init(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    // For SQLite, ensure the database file and parent directory exist
    if database_url.starts_with("sqlite://") {
        let db_path = database_url.trim_start_matches("sqlite://");

        // Create parent directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(db_path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| DbErr::Custom(format!("Failed to create database directory: {}", e)))?;
            }
        }

        // Create empty database file if it doesn't exist
        if !std::path::Path::new(db_path).exists() {
            std::fs::File::create(db_path)
                .map_err(|e| DbErr::Custom(format!("Failed to create database file: {}", e)))?;
        }
    }

    let mut opt = ConnectOptions::new(database_url);
    opt.sqlx_logging(false);

    if database_url.starts_with("sqlite") {
        // Single connection for SQLite to avoid write-lock contention; WAL mode
        // is enabled via migration pragma for SD card longevity on Raspberry Pi.
        opt.min_connections(1).max_connections(1);
    } else {
        opt.min_connections(2).max_connections(20);
    }

    Database::connect(opt).await
}

/// Apply all pending migrations.
pub async fn run_migrations(db: &DatabaseConnection) -> Result<(), DbErr> {
    migrations::Migrator::up(db, None).await
}
