use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_employee_table(manager).await?;
        create_employee_refresh_token_table(manager).await?;
        create_employee_email_verification_token_table(manager).await?;
        create_employee_email_verification_token_index(manager).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(EmployeeEmailVerificationToken::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(EmployeeRefreshToken::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Employee::Table).to_owned())
            .await?;

        Ok(())
    }
}

async fn create_employee_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Employee::Table)
                .col(ColumnDef::new(Employee::Id).text().not_null().primary_key())
                .col(
                    ColumnDef::new(Employee::Email)
                        .string_len(255)
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
                .col(ColumnDef::new(Employee::EmailVerifiedAt).text().null())
                .col(
                    ColumnDef::new(Employee::CreatedAt)
                        .text()
                        .not_null()
                        .default("CURRENT_TIMESTAMP"),
                )
                .col(
                    ColumnDef::new(Employee::UpdatedAt)
                        .text()
                        .not_null()
                        .default("CURRENT_TIMESTAMP"),
                )
                .to_owned(),
        )
        .await
}

async fn create_employee_refresh_token_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(EmployeeRefreshToken::Table)
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
                        .string_len(255)
                        .not_null()
                        .unique_key(),
                )
                .col(
                    ColumnDef::new(EmployeeRefreshToken::ExpiresAt)
                        .text()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(EmployeeRefreshToken::RevokedAt)
                        .text()
                        .null(),
                )
                .col(
                    ColumnDef::new(EmployeeRefreshToken::CreatedAt)
                        .text()
                        .not_null()
                        .default("CURRENT_TIMESTAMP"),
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(
                            EmployeeRefreshToken::Table,
                            EmployeeRefreshToken::EmployeeId,
                        )
                        .to(Employee::Table, Employee::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await
}

async fn create_employee_email_verification_token_table(
    manager: &SchemaManager<'_>,
) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(EmployeeEmailVerificationToken::Table)
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
                        .string_len(255)
                        .not_null()
                        .unique_key(),
                )
                .col(
                    ColumnDef::new(EmployeeEmailVerificationToken::ExpiresAt)
                        .text()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(EmployeeEmailVerificationToken::CreatedAt)
                        .text()
                        .not_null()
                        .default("CURRENT_TIMESTAMP"),
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(
                            EmployeeEmailVerificationToken::Table,
                            EmployeeEmailVerificationToken::EmployeeId,
                        )
                        .to(Employee::Table, Employee::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await
}

async fn create_employee_email_verification_token_index(
    manager: &SchemaManager<'_>,
) -> Result<(), DbErr> {
    manager
        .create_index(
            Index::create()
                .name("idx_employee_email_verification_tokens_employee_id")
                .table(EmployeeEmailVerificationToken::Table)
                .col(EmployeeEmailVerificationToken::EmployeeId)
                .to_owned(),
        )
        .await
}

#[derive(DeriveIden)]
enum Employee {
    Table,
    Id,
    Email,
    PasswordHash,
    Role,
    IsActive,
    EmailVerifiedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum EmployeeRefreshToken {
    Table,
    Id,
    EmployeeId,
    TokenHash,
    ExpiresAt,
    RevokedAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum EmployeeEmailVerificationToken {
    Table,
    Id,
    EmployeeId,
    TokenHash,
    ExpiresAt,
    CreatedAt,
}
