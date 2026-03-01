use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "migration_000004_create_request_logs"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RequestLogs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RequestLogs::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(RequestLogs::TokenId).big_integer().null())
                    .col(ColumnDef::new(RequestLogs::UpstreamId).big_integer().null())
                    .col(ColumnDef::new(RequestLogs::Path).string().not_null())
                    .col(ColumnDef::new(RequestLogs::Method).string().not_null())
                    .col(ColumnDef::new(RequestLogs::StatusCode).integer().not_null())
                    .col(
                        ColumnDef::new(RequestLogs::LatencyMs)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RequestLogs::RequestSize)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RequestLogs::ResponseSize)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RequestLogs::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Index for time-based range queries
        manager
            .create_index(
                Index::create()
                    .table(RequestLogs::Table)
                    .name("idx_request_logs_created_at")
                    .col(RequestLogs::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // Index for per-token statistics
        manager
            .create_index(
                Index::create()
                    .table(RequestLogs::Table)
                    .name("idx_request_logs_token_id")
                    .col(RequestLogs::TokenId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RequestLogs::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum RequestLogs {
    Table,
    Id,
    TokenId,
    UpstreamId,
    Path,
    Method,
    StatusCode,
    LatencyMs,
    RequestSize,
    ResponseSize,
    CreatedAt,
}
