# Generate .sql files from SeaQuery migration definitions
generate-migration:
    cargo run -p migration -- --output migrations/

# Run pending migrations via sqlx-cli
up-migration:
    sqlx migrate run --source migrations/

# Revert last migration
down-migration:
    sqlx migrate revert --source migrations/

# Revert all migrations
reset-migration:
    sqlx migrate revert --source migrations/ --target-version 0

# Check migration status
migrate-status:
    sqlx migrate info --source migrations/

# Create a new migration (interactive)
migrate-create name:
    #!/usr/bin/env bash
    set -euo pipefail
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    MIGRATION_NAME="${TIMESTAMP}_{{name}}"
    MIGRATION_FILE="migration/src/${MIGRATION_NAME}.rs"
    echo "Creating migration: ${MIGRATION_NAME}"

    cat > "${MIGRATION_FILE}" << 'RS_EOF'
    use sea_query::{SqliteQueryBuilder, Table};

    pub fn up() -> Vec<String> {
        vec![]
    }

    pub fn down() -> Vec<String> {
        vec![]
    }
    RS_EOF

    echo "Created ${MIGRATION_FILE}"
    echo "Don't forget to add the migration to migration/src/lib.rs"

# Run test server with release profile (for integration/performance testing via Hurl)
test-server:
    cargo run --features integration_testing

test-server-release:
    cargo run --release --features integration_testing

# Run cargo check (both default and integration_testing features)
check:
    cargo check
    cargo check --features integration_testing

# Run cargo fmt
fmt:
    cargo fmt
