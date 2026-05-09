use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub email: String,
    #[sea_orm(column_type = "Text")]
    pub password_hash: String,
    #[sea_orm(column_type = "Text")]
    pub role: String,
    pub is_active: bool,
    pub email_verified_at: Option<DateTimeWithTimeZone>,
    pub updated_at: DateTime,
    pub created_at: DateTime,
}

impl ActiveModelBehavior for ActiveModel {}
