use sea_orm_migration::prelude::*;

mod migration_000001_create_upstreams;
mod migration_000002_create_api_tokens;
mod migration_000003_create_route_rules;
mod migration_000004_create_request_logs;
mod migration_000005_create_health_records;
mod migration_000006_add_upstream_ids;

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(migration_000001_create_upstreams::Migration),
            Box::new(migration_000002_create_api_tokens::Migration),
            Box::new(migration_000003_create_route_rules::Migration),
            Box::new(migration_000004_create_request_logs::Migration),
            Box::new(migration_000005_create_health_records::Migration),
            Box::new(migration_000006_add_upstream_ids::Migration),
        ]
    }
}
