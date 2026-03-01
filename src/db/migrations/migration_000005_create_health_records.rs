use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "migration_000005_create_health_records"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(HealthRecords::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(HealthRecords::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(HealthRecords::UpstreamId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(HealthRecords::CheckedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(HealthRecords::Success).boolean().not_null())
                    .col(
                        ColumnDef::new(HealthRecords::LatencyMs)
                            .big_integer()
                            .null(),
                    )
                    .col(ColumnDef::new(HealthRecords::ErrorMessage).text().null())
                    .to_owned(),
            )
            .await?;

        // Index for querying health history per upstream
        manager
            .create_index(
                Index::create()
                    .table(HealthRecords::Table)
                    .name("idx_health_records_upstream_id")
                    .col(HealthRecords::UpstreamId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(HealthRecords::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum HealthRecords {
    Table,
    Id,
    UpstreamId,
    CheckedAt,
    Success,
    LatencyMs,
    ErrorMessage,
}
