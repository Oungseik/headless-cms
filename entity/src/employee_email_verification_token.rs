use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

/// Email verification token model.
#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "employee_email_verification_token")]
pub struct Model {
    /// Primary key — UUID v7 stored as text.
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// FK to employee.
    pub employee_id: Uuid,
    /// SHA-256 hash of the raw token.
    pub token_hash: String,
    /// Token expiry timestamp.
    pub expires_at: DateTime<Utc>,
    /// Row creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Email verification token entity (table definition).
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Belongs to an employee.
    #[sea_orm(
        belongs_to = "super::employee::Entity",
        from = "Column::EmployeeId",
        to = "super::employee::Column::Id",
        on_delete = "Cascade"
    )]
    Employee,
}

impl Related<super::employee::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Employee.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
