pub use sea_orm_migration::prelude::*;

mod m20260502_000001_create_user_table;
mod m20260509_000002_auth_tables;

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260502_000001_create_user_table::Migration),
            Box::new(m20260509_000002_auth_tables::Migration),
        ]
    }
}
