use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;

pub mod entities;
pub mod migrations;

/// Initialize the database connection pool.
pub async fn init(database_url: &str) -> Result<DatabaseConnection, DbErr> {
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
