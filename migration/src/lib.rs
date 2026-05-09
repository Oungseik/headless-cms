pub use sea_orm_migration::prelude::*;

mod m20260509_000001_initial_schema;

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20260509_000001_initial_schema::Migration)]
    }
}
