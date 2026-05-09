use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop username column from user table
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .drop_column(User::Username)
                    .to_owned(),
            )
            .await?;

        // Create unique index on email
        manager
            .create_index(
                Index::create()
                    .name("idx_user_email")
                    .table(User::Table)
                    .col(User::Email)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Add email_verified_at column
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .add_column(ColumnDef::new(User::EmailVerifiedAt).timestamp_with_time_zone())
                    .to_owned(),
            )
            .await?;

        // Create email_verification_tokens table
        manager
            .create_table(
                Table::create()
                    .table(EmailVerificationToken::Table)
                    .if_not_exists()
                    .col(pk_auto(EmailVerificationToken::Id))
                    .col(integer(EmailVerificationToken::UserId))
                    .col(string_uniq(EmailVerificationToken::TokenHash))
                    .col(
                        ColumnDef::new(EmailVerificationToken::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(EmailVerificationToken::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_email_verification_tokens_user_id")
                            .from(
                                EmailVerificationToken::Table,
                                EmailVerificationToken::UserId,
                            )
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on user_id for delete queries
        manager
            .create_index(
                Index::create()
                    .name("idx_email_verification_tokens_user_id")
                    .table(EmailVerificationToken::Table)
                    .col(EmailVerificationToken::UserId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop user_id index
        manager
            .drop_index(
                Index::drop()
                    .name("idx_email_verification_tokens_user_id")
                    .table(EmailVerificationToken::Table)
                    .to_owned(),
            )
            .await?;

        // Drop email_verification_tokens table
        manager
            .drop_table(
                Table::drop()
                    .table(EmailVerificationToken::Table)
                    .to_owned(),
            )
            .await?;

        // Drop unique index on email
        manager
            .drop_index(
                Index::drop()
                    .name("idx_user_email")
                    .table(User::Table)
                    .to_owned(),
            )
            .await?;

        // Drop email_verified_at column
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .drop_column(User::EmailVerifiedAt)
                    .to_owned(),
            )
            .await?;

        // Re-add username column
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .add_column(
                        ColumnDef::new(User::Username)
                            .text()
                            .unique_key()
                            .not_null()
                            .default(""),
                    )
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Username,
    Email,
    EmailVerifiedAt,
}

#[derive(DeriveIden)]
enum EmailVerificationToken {
    Table,
    Id,
    UserId,
    TokenHash,
    ExpiresAt,
    CreatedAt,
}
