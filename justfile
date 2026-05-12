# Run pending migrations
migrate-run:
    sqlx migrate run

# Create a new migration
migrate-create name:
    sqlx migrate add {{name}}

# Generate offline query metadata for compile-time checks
migrate-prepare:
    cargo sqlx prepare

# Revert the last migration
migrate-revert:
    sqlx migrate revert

# Run test server with release profile (for integration/performance testing via Hurl)
test-server:
    DATABASE_URL="sqlite::memory:?cache=shared" cargo run

test-server-release:
    DATABASE_URL="sqlite::memory:?cache=shared" cargo run --release

# Run cargo check
check:
    cargo check

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
