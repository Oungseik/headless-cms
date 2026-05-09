use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Alter user table: add auth columns
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .add_column(
                        ColumnDef::new(User::PasswordHash)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .add_column(
                        ColumnDef::new(User::Role)
                            .text()
                            .not_null()
                            .default("customer"),
                    )
                    .add_column(
                        ColumnDef::new(User::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .add_column(
                        ColumnDef::new(User::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create refresh_tokens table
        manager
            .create_table(
                Table::create()
                    .table(RefreshToken::Table)
                    .if_not_exists()
                    .col(pk_auto(RefreshToken::Id))
                    .col(integer(RefreshToken::UserId))
                    .col(string_uniq(RefreshToken::TokenHash))
                    .col(timestamp(RefreshToken::ExpiresAt))
                    .col(ColumnDef::new(RefreshToken::RevokedAt).timestamp().null())
                    .col(
                        ColumnDef::new(RefreshToken::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_refresh_tokens_user_id")
                            .from(RefreshToken::Table, RefreshToken::UserId)
                            .to(User::Table, User::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RefreshToken::Table).to_owned())
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .drop_column(User::PasswordHash)
                    .drop_column(User::Role)
                    .drop_column(User::IsActive)
                    .drop_column(User::UpdatedAt)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    PasswordHash,
    Role,
    IsActive,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum RefreshToken {
    Table,
    Id,
    UserId,
    TokenHash,
    ExpiresAt,
    RevokedAt,
    CreatedAt,
}
