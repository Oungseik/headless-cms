# POS Backend

A headless POS + CMS backend built with Axum, sqlx, and preconfigured OpenAPI, OpenTelemetry.

## Tech Stack

- [**Axum**](https://github.com/tokio-rs/axum) -- async web framework
- [**sqlx**](https://github.com/launchbadge/sqlx) -- async SQL toolkit (SQLite)
- [**Utoipa**](https://github.com/juhaku/utoipa) -- OpenAPI spec generation + Swagger UI
- [**OpenTelemetry**](https://opentelemetry.io/) -- distributed tracing (OTLP export)
- [**tower_governor**](https://github.com/benwis/tower_governor) -- rate limiting
- **jsonwebtoken** + **bcrypt** -- JWT auth and password hashing

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/health/` | Health check |
| `POST` | `/api/v1/dashboard/auth/register` | Register first owner account |
| `POST` | `/api/v1/dashboard/auth/login` | Login, returns JWT + refresh token |
| `POST` | `/api/v1/dashboard/auth/test/verify-all` | Test-only: verify all emails (when `APP_ENV=testing`) |

Swagger UI at `/api-docs/swagger-ui`.

## Getting Started

Prerequisites: Rust toolchain, SQLite. Optionally use [Nix](https://nixos.org/) for a reproducible dev shell.

```sh
nix develop          # optional, provides all tools
just migrate-run     # run pending migrations
cargo run            # start the server
```

## Testing

```sh
just verify          # format + lint + tests (full CI)
cargo test --all     # unit tests only
```

For E2E tests with [Hurl](https://hurl.dev/):

```sh
just test-server                                    # terminal 1: in-memory SQLite server
hurl --test tests/hurl_e2e/**/*.hurl                # terminal 2: run E2E tests
```

## Dev Commands

| Command | Description |
|---------|-------------|
| `just verify` | Format + lint + test |
| `just test` | Run all tests |
| `just lint` | Clippy strict |
| `just fmt` | Format code |
| `just check` | Cargo check |
| `just migrate-run` | Run pending migrations |
| `just migrate-create <name>` | Create a new migration |
| `just migrate-revert` | Revert last migration |
| `just test-server` | Start in-memory server for E2E |

## Configuration

Config loaded in [config.rs](./src/config.rs) via `clap` from `.env`, env vars, and CLI flags.

Key settings: `APP_ENV`, `DATABASE_URL`, `JWT_SECRET`, `LISTEN_ADDR`, `ALLOWED_ORIGINS`, `RATE_LIMIT_ENABLED`, `BCRYPT_COST`.

Traces via [otel-desktop-viewer](https://github.com/CtrlSpice/otel-desktop-viewer) or any OTLP backend.
