use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "migration_000003_create_route_rules"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RouteRules::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RouteRules::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(RouteRules::Name).string().not_null())
                    .col(ColumnDef::new(RouteRules::InboundPath).string().not_null())
                    .col(ColumnDef::new(RouteRules::OutboundPath).string().not_null())
                    .col(
                        ColumnDef::new(RouteRules::MatchType)
                            .string()
                            .not_null()
                            .default("exact"),
                    )
                    .col(ColumnDef::new(RouteRules::UpstreamId).big_integer().null())
                    .col(
                        ColumnDef::new(RouteRules::Priority)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(RouteRules::ExtraHeaders).text().null())
                    .col(ColumnDef::new(RouteRules::ExtraQuery).text().null())
                    .col(
                        ColumnDef::new(RouteRules::Enabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(RouteRules::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RouteRules::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RouteRules::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum RouteRules {
    Table,
    Id,
    Name,
    InboundPath,
    OutboundPath,
    MatchType,
    UpstreamId,
    Priority,
    ExtraHeaders,
    ExtraQuery,
    Enabled,
    CreatedAt,
    UpdatedAt,
}
