use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

/// Employee account model.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "employee")]
pub struct Model {
    /// Primary key — UUID v7 stored as text.
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// Unique email address.
    pub email: String,
    /// Bcrypt password hash.
    pub password_hash: String,
    /// Role (e.g. `owner`).
    pub role: String,
    /// Whether the account is active.
    pub is_active: bool,
    /// Timestamp when email was verified, if ever.
    pub email_verified_at: Option<DateTime<Utc>>,
    /// Row creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Row last-updated timestamp.
    pub updated_at: DateTime<Utc>,
}

/// Employee entity (table definition).
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// An employee has many refresh tokens.
    #[sea_orm(has_many = "super::employee_refresh_token::Entity")]
    EmployeeRefreshToken,
    /// An employee has many email verification tokens.
    #[sea_orm(has_many = "super::employee_email_verification_token::Entity")]
    EmployeeEmailVerificationToken,
}

impl Related<super::employee_refresh_token::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EmployeeRefreshToken.def()
    }
}

impl Related<super::employee_email_verification_token::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EmployeeEmailVerificationToken.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
