use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");

    let db = sea_orm::Database::connect(&database_url)
        .await
        .expect("failed to connect to database");

    migration::Migrator::up(&db, None)
        .await
        .expect("failed to run migrations");
}
