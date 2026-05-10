use sea_query::{ColumnDef, Expr, ForeignKey, ForeignKeyAction, Index, SqliteQueryBuilder, Table};

use entity::employee::Employee;
use entity::employee_email_verification_token::EmployeeEmailVerificationToken;
use entity::employee_refresh_token::EmployeeRefreshToken;

fn create_employee_table() -> String {
    Table::create()
        .table(Employee::Table)
        .if_not_exists()
        .col(ColumnDef::new(Employee::Id).text().not_null().primary_key())
        .col(
            ColumnDef::new(Employee::Email)
                .string()
                .not_null()
                .unique_key(),
        )
        .col(
            ColumnDef::new(Employee::PasswordHash)
                .text()
                .not_null()
                .default(""),
        )
        .col(
            ColumnDef::new(Employee::Role)
                .text()
                .not_null()
                .default("owner"),
        )
        .col(
            ColumnDef::new(Employee::IsActive)
                .boolean()
                .not_null()
                .default(true),
        )
        .col(ColumnDef::new(Employee::EmailVerifiedAt).timestamp_with_time_zone())
        .col(
            ColumnDef::new(Employee::CreatedAt)
                .timestamp_with_time_zone()
                .not_null()
                .default(Expr::current_timestamp()),
        )
        .col(
            ColumnDef::new(Employee::UpdatedAt)
                .timestamp_with_time_zone()
                .not_null()
                .default(Expr::current_timestamp()),
        )
        .build(SqliteQueryBuilder)
}

fn create_employee_refresh_tokens_table() -> String {
    Table::create()
        .table(EmployeeRefreshToken::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(EmployeeRefreshToken::Id)
                .text()
                .not_null()
                .primary_key(),
        )
        .col(
            ColumnDef::new(EmployeeRefreshToken::EmployeeId)
                .text()
                .not_null(),
        )
        .col(
            ColumnDef::new(EmployeeRefreshToken::TokenHash)
                .string()
                .not_null()
                .unique_key(),
        )
        .col(
            ColumnDef::new(EmployeeRefreshToken::ExpiresAt)
                .timestamp_with_time_zone()
                .not_null(),
        )
        .col(ColumnDef::new(EmployeeRefreshToken::RevokedAt).timestamp_with_time_zone())
        .col(
            ColumnDef::new(EmployeeRefreshToken::CreatedAt)
                .timestamp_with_time_zone()
                .not_null()
                .default(Expr::current_timestamp()),
        )
        .foreign_key(
            ForeignKey::create()
                .name("fk_employee_refresh_tokens_employee_id")
                .from(
                    EmployeeRefreshToken::Table,
                    EmployeeRefreshToken::EmployeeId,
                )
                .to(Employee::Table, Employee::Id)
                .on_delete(ForeignKeyAction::Cascade),
        )
        .build(SqliteQueryBuilder)
}

fn create_employee_email_verification_tokens_table() -> String {
    Table::create()
        .table(EmployeeEmailVerificationToken::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(EmployeeEmailVerificationToken::Id)
                .text()
                .not_null()
                .primary_key(),
        )
        .col(
            ColumnDef::new(EmployeeEmailVerificationToken::EmployeeId)
                .text()
                .not_null(),
        )
        .col(
            ColumnDef::new(EmployeeEmailVerificationToken::TokenHash)
                .string()
                .not_null()
                .unique_key(),
        )
        .col(
            ColumnDef::new(EmployeeEmailVerificationToken::ExpiresAt)
                .timestamp_with_time_zone()
                .not_null(),
        )
        .col(
            ColumnDef::new(EmployeeEmailVerificationToken::CreatedAt)
                .timestamp_with_time_zone()
                .not_null()
                .default(Expr::current_timestamp()),
        )
        .foreign_key(
            ForeignKey::create()
                .name("fk_employee_email_verification_tokens_employee_id")
                .from(
                    EmployeeEmailVerificationToken::Table,
                    EmployeeEmailVerificationToken::EmployeeId,
                )
                .to(Employee::Table, Employee::Id)
                .on_delete(ForeignKeyAction::Cascade),
        )
        .build(SqliteQueryBuilder)
}

fn create_employee_email_verification_tokens_index() -> String {
    Index::create()
        .name("idx_employee_email_verification_tokens_employee_id")
        .table(EmployeeEmailVerificationToken::Table)
        .col(EmployeeEmailVerificationToken::EmployeeId)
        .build(SqliteQueryBuilder)
}

pub fn up() -> Vec<String> {
    vec![
        create_employee_table(),
        create_employee_refresh_tokens_table(),
        create_employee_email_verification_tokens_table(),
        create_employee_email_verification_tokens_index(),
    ]
}

pub fn down() -> Vec<String> {
    vec![
        Table::drop()
            .table(EmployeeEmailVerificationToken::Table)
            .build(SqliteQueryBuilder),
        Table::drop()
            .table(EmployeeRefreshToken::Table)
            .build(SqliteQueryBuilder),
        Table::drop()
            .table(Employee::Table)
            .build(SqliteQueryBuilder),
    ]
}
