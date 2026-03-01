use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "migration_000002_create_api_tokens"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ApiTokens::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ApiTokens::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ApiTokens::TokenHash)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(ApiTokens::Label).string().not_null())
                    .col(
                        ColumnDef::new(ApiTokens::ExpiresAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ApiTokens::Enabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(ColumnDef::new(ApiTokens::RpmLimit).integer().null())
                    .col(ColumnDef::new(ApiTokens::TpmLimit).integer().null())
                    .col(
                        ColumnDef::new(ApiTokens::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ApiTokens::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ApiTokens::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum ApiTokens {
    Table,
    Id,
    TokenHash,
    Label,
    ExpiresAt,
    Enabled,
    RpmLimit,
    TpmLimit,
    CreatedAt,
    UpdatedAt,
}
