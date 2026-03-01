use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "migration_000001_create_upstreams"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Upstreams::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Upstreams::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Upstreams::Name)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Upstreams::BaseUrl).string().not_null())
                    .col(ColumnDef::new(Upstreams::ApiKey).string().null())
                    .col(ColumnDef::new(Upstreams::ExtraHeaders).text().null())
                    .col(
                        ColumnDef::new(Upstreams::TimeoutSecs)
                            .integer()
                            .not_null()
                            .default(30),
                    )
                    .col(
                        ColumnDef::new(Upstreams::Weight)
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .col(
                        ColumnDef::new(Upstreams::Priority)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Upstreams::LbStrategy)
                            .string()
                            .not_null()
                            .default("round_robin"),
                    )
                    .col(
                        ColumnDef::new(Upstreams::Enabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Upstreams::IsHealthy)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Upstreams::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Upstreams::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Upstreams::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Upstreams {
    Table,
    Id,
    Name,
    BaseUrl,
    ApiKey,
    ExtraHeaders,
    TimeoutSecs,
    Weight,
    Priority,
    LbStrategy,
    Enabled,
    IsHealthy,
    CreatedAt,
    UpdatedAt,
}
