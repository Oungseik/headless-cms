use sea_query::{ColumnDef, Expr, ForeignKey, ForeignKeyAction, Index, SqliteQueryBuilder, Table};

use entity::email_verification_token::EmailVerificationToken;
use entity::refresh_token::RefreshToken;
use entity::user::User;

fn create_user_table() -> String {
    Table::create()
        .table(User::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(User::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(User::Email).string().not_null().unique_key())
        .col(
            ColumnDef::new(User::PasswordHash)
                .text()
                .not_null()
                .default(""),
        )
        .col(
            ColumnDef::new(User::Role)
                .text()
                .not_null()
                .default("customer"),
        )
        .col(
            ColumnDef::new(User::IsActive)
                .boolean()
                .not_null()
                .default(true),
        )
        .col(ColumnDef::new(User::EmailVerifiedAt).timestamp_with_time_zone())
        .col(
            ColumnDef::new(User::CreatedAt)
                .timestamp()
                .not_null()
                .default(Expr::current_timestamp()),
        )
        .col(
            ColumnDef::new(User::UpdatedAt)
                .timestamp()
                .not_null()
                .default(Expr::current_timestamp()),
        )
        .build(SqliteQueryBuilder)
}

fn create_refresh_tokens_table() -> String {
    Table::create()
        .table(RefreshToken::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(RefreshToken::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(RefreshToken::UserId).integer().not_null())
        .col(
            ColumnDef::new(RefreshToken::TokenHash)
                .string()
                .not_null()
                .unique_key(),
        )
        .col(
            ColumnDef::new(RefreshToken::ExpiresAt)
                .timestamp()
                .not_null(),
        )
        .col(ColumnDef::new(RefreshToken::RevokedAt).timestamp())
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
        .build(SqliteQueryBuilder)
}

fn create_email_verification_tokens_table() -> String {
    Table::create()
        .table(EmailVerificationToken::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(EmailVerificationToken::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(EmailVerificationToken::UserId)
                .integer()
                .not_null(),
        )
        .col(
            ColumnDef::new(EmailVerificationToken::TokenHash)
                .string()
                .not_null()
                .unique_key(),
        )
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
        .build(SqliteQueryBuilder)
}

fn create_email_verification_tokens_index() -> String {
    Index::create()
        .name("idx_email_verification_tokens_user_id")
        .table(EmailVerificationToken::Table)
        .col(EmailVerificationToken::UserId)
        .build(SqliteQueryBuilder)
}

pub fn up() -> Vec<String> {
    vec![
        create_user_table(),
        create_refresh_tokens_table(),
        create_email_verification_tokens_table(),
        create_email_verification_tokens_index(),
    ]
}

pub fn down() -> Vec<String> {
    vec![
        Table::drop()
            .table(EmailVerificationToken::Table)
            .build(SqliteQueryBuilder),
        Table::drop()
            .table(RefreshToken::Table)
            .build(SqliteQueryBuilder),
        Table::drop().table(User::Table).build(SqliteQueryBuilder),
    ]
}
