# Run pending migrations via SeaORM
up-migration:
    cargo run -p migration

# Create a new migration
migrate-create name:
    #!/usr/bin/env bash
    set -euo pipefail
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    MIGRATION_NAME="${TIMESTAMP}_{{name}}"
    MIGRATION_FILE="migration/src/${MIGRATION_NAME}.rs"
    echo "Creating migration: ${MIGRATION_NAME}"

    cat > "${MIGRATION_FILE}" << 'RS_EOF'
    use sea_orm_migration::prelude::*;

    #[derive(DeriveMigrationName)]
    pub struct Migration;

    #[async_trait::async_trait]
    impl MigrationTrait for Migration {
        async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            Ok(())
        }

        async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            Ok(())
        }
    }
    RS_EOF

    echo "Created ${MIGRATION_FILE}"
    echo "Don't forget to add the migration to migration/src/lib.rs"

# Run test server with release profile (for integration/performance testing via Hurl)
test-server:
    DATABASE_URL="sqlite::memory:?cache=shared" cargo run 

test-server-release:
    DATABASE_URL="sqlite::memory:?cache=shared" cargo run --release 

# Run cargo check (both default and integration_testing features)
check:
    cargo check
    cargo check --features integration_testing

# Run cargo fmt
fmt:
    cargo fmt

# Run clippy with strict warnings across all targets and features
lint:
    cargo clippy --all-targets --all-features --locked -- -D warnings

# Run all tests
test:
    cargo test --all

# Run format check + lint + tests
verify: fmt lint test
