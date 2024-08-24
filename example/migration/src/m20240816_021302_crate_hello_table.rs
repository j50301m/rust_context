use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        self.create_table(manager).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Hello::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Hello {
    Table,
    Id,
    Name,
    CreateAt,
    UpdateAt,
}

impl Migration {
    async fn create_table(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Hello::Table)
                    .if_not_exists()
                    .col(big_integer(Hello::Id).primary_key().auto_increment())
                    .col(string(Hello::Name).not_null())
                    .col(
                        timestamp(Hello::CreateAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp(Hello::UpdateAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        let comment = r#"
            COMMENT ON TABLE hello IS '錢包來源';
            COMMENT ON COLUMN hello.id IS 'ID';
            COMMENT ON COLUMN hello.name IS '名稱';
            COMMENT ON COLUMN hello.create_at IS '建立時間';
            COMMENT ON COLUMN hello.update_at IS '更新時間';
        "#;

        manager.get_connection().execute_unprepared(comment).await?;
        Ok(())
    }
}
