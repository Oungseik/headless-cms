# Conventions

Living reference for adding features to this Rust/Axum/SeaORM POS + CMS backend.

## Architecture Overview

```
config → app → features → services → entities
```

| Layer | Responsibility | Location |
|-------|---------------|----------|
| **Config** | Environment variables, database URL | `src/config.rs` |
| **App** | AppState, router wiring, DB setup, error types | `src/app.rs`, `src/app/error.rs` |
| **Features** | Domain handlers and routers | `src/features/<domain>/` |
| **Services** | Trait + impl decoupling handlers from DB | `src/features/<domain>/service*.rs` |
| **Entities** | SeaORM generated models | `entity/` crate |
| **Migrations** | Database schema versions | `migration/` crate |

AppState holds `Arc<dyn <Domain>Service>` for each domain, enabling test-time substitution via mocks.

## Feature Directory Structure

```
src/features/<domain>/
  mod.rs           — pub fn router() -> OpenApiRouter<Arc<AppState>>
  service.rs       — #[async_trait] trait definition
  service_impl.rs  — production implementation (wraps DatabaseConnection)
  service_mock.rs  — #[cfg(test)] mock with interior mutability
  <operation>.rs   — one file per handler (e.g. get_by_id.rs, create.rs)
```

Simple domains without a service layer (like health) can be a single file at `src/features/<domain>.rs` with a directory for handlers alongside it.

## Adding a New Domain

### 1. Create entity and migration (if needed)

Use SeaORM CLI to generate from the database, or hand-write in the `entity/` and `migration/` crates.

### 2. Define the service trait

File: `src/features/<domain>/service.rs`

```rust
use async_trait::async_trait;

#[derive(Debug)]
pub enum PostServiceError {
    NotFound(i32),
    Database(sea_orm::DbErr),
}

#[async_trait]
pub trait PostService: Send + Sync + 'static {
    async fn get_by_id(
        &self,
        id: i32,
    ) -> Result<Option<entity::post::Model>, PostServiceError>;
}
```

Convention: `Send + Sync + 'static` bounds are required so the trait can be wrapped in `Arc<dyn>` and shared across threads.

### 3. Create the production implementation

File: `src/features/<domain>/service_impl.rs`

```rust
use async_trait::async_trait;
use sea_orm::{DatabaseConnection, EntityTrait};

use super::service::{PostService, PostServiceError};

#[derive(Clone, Debug)]
pub struct PostServiceImpl {
    pub db: DatabaseConnection,
}

#[async_trait]
impl PostService for PostServiceImpl {
    async fn get_by_id(
        &self,
        id: i32,
    ) -> Result<Option<entity::post::Model>, PostServiceError> {
        entity::post::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(PostServiceError::Database)
    }
}
```

### 4. Add field to AppState

File: `src/app.rs` — add to the struct and Debug impl:

```rust
#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<dyn UserService>,
    pub post_service: Arc<dyn PostService>,  // new
}
```

> **Note:** `DatabaseConnection` is NOT a field on `AppState`. It lives only inside each `*ServiceImpl`, which is constructed in `create_app()` and then erased behind `Arc<dyn <Domain>Service>`.

Add the import at the top:

```rust
use crate::features::posts::service::PostService;
```

### 5. Wire in create_app()

File: `src/app.rs` — construct the impl and pass to state:

```rust
let post_service: Arc<dyn PostService> =
    Arc::new(crate::features::posts::service_impl::PostServiceImpl { db: db.clone() });
let state = Arc::new(AppState { user_service, post_service });
```

### 6. Create handler files

File: `src/features/<domain>/<operation>.rs`

```rust
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use serde::Serialize;
use utoipa::ToSchema;

use crate::app::AppState;
use crate::app::error::{AppError, AppResult, ErrorResponse};

#[derive(Serialize, ToSchema)]
pub struct PostResponse {
    pub id: i32,
    pub title: String,
}

impl From<entity::post::Model> for PostResponse {
    fn from(model: entity::post::Model) -> Self {
        Self { id: model.id, title: model.title }
    }
}

#[utoipa::path(
    get,
    path = "/{id}",
    description = "Get a post by ID",
    responses(
        (status = 200, description = "Post found", body = PostResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    tag = "Posts",
)]
#[tracing::instrument]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> AppResult<Json<PostResponse>> {
    let post = state
        .post_service
        .get_by_id(id)
        .await
        .map_err(AppError::from)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(PostResponse::from(post)))
}
```

Every handler follows this shape: extract state/path/body, call service, map errors via `AppError::from` / `.ok_or(AppError::NotFound)`, return `AppResult<Json<T>>`.

> **Note:** `AppError::from` works because `error.rs` contains a `From<PostServiceError> for AppError` impl. Add a matching `From` impl for each new domain error type.

### 7. Create the router

File: `src/features/<domain>/mod.rs`

```rust
pub mod get_by_id;
pub mod service;
pub mod service_impl;

#[cfg(test)]
pub mod service_mock;

use std::sync::Arc;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::app::AppState;

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new().routes(routes!(get_by_id::handler))
}
```

### 8. Register routes in app.rs

File: `src/app.rs` — add the route nest:

```rust
let posts_route = features::posts::router();

let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
    .nest("/health", health_route)
    .nest("/api/v1/users", users_route)
    .nest("/api/v1/posts", posts_route)
    .with_state(state)
    .split_for_parts();
```

### 9. Create the mock service

File: `src/features/<domain>/service_mock.rs`

```rust
#[cfg(test)]
pub mod tests {
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    use crate::features::posts::service::{PostService, PostServiceError};

    #[derive(Debug)]
    pub struct MockPostService {
        pub posts: Mutex<HashMap<i32, entity::post::Model>>,
    }

    impl MockPostService {
        pub fn new() -> Self {
            Self { posts: Mutex::new(HashMap::new()) }
        }
    }

    #[async_trait]
    impl PostService for MockPostService {
        async fn get_by_id(
            &self,
            id: i32,
        ) -> Result<Option<entity::post::Model>, PostServiceError> {
            let posts = self.posts.lock().unwrap();
            Ok(posts.get(&id).cloned())
        }
    }
}
```

### 10. Write tests inline in handler files

Tests go in `#[cfg(test)] mod tests` at the bottom of the handler file. See "Testing Pattern" below.

## Adding an Operation to an Existing Domain

To add `create_post` to the posts domain:

1. **Add method to service trait** (`service.rs`)
2. **Implement in service_impl** (`service_impl.rs`)
3. **Add method to mock** (`service_mock.rs`)
4. **Create handler file** (`create.rs`) with `UserResponse`/`PostResponse` or a new `<Operation>Response` type
5. **Register in router** (`mod.rs`) — add `pub mod create;` and `.routes(routes!(create::handler))`
6. **Add utoipa path annotation** with request body and response types
7. **Write tests** in the handler file

No changes to `app.rs` or `AppState` unless the operation requires a new service dependency.

## Naming Conventions

| Item | Convention | Example |
|------|-----------|---------|
| Service trait | `<Domain>Service` | `UserService` |
| Production impl | `<Domain>ServiceImpl` | `UserServiceImpl` |
| Mock impl | `Mock<Domain>Service` | `MockUserService` |
| Handler file | `<operation>.rs` (snake_case) | `get_by_id.rs`, `create.rs` |
| Response type | `<Domain>Response` | `UserResponse` |
| Handler function | `pub async fn handler` | always `handler` — the file name distinguishes operations |
| Route path | `/api/v1/<domain>` | `/api/v1/users` |
| Tag (utoipa) | Plural PascalCase | `"Users"`, `"Posts"` |

## Testing Pattern

Tests live inline at the bottom of each handler file:

```rust
#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use axum::extract::{Path, State};
    use crate::app::AppState;
    use crate::features::users::service_mock::tests::MockUserService;
    use crate::features::users::service::UserService;

    #[tokio::test]
    async fn test_get_user_by_id_found() {
        // 1. Build mock and seed data
        let mock = MockUserService::new();
        mock.users.lock().unwrap().insert(1, entity::user::Model { /* ... */ });

        // 2. Wrap in Arc<dyn Trait>
        let mock: Arc<dyn UserService> = Arc::new(mock);

        // 3. Create AppState with mock
        let state = Arc::new(AppState { user_service: mock });

        // 4. Call handler directly
        let result = super::handler(State(state), Path(1)).await;

        // 5. Assert
        assert!(result.is_ok());
    }
}
```

Key points:

- Use `MockUserService` for unit tests — no real database needed
- `AppState` has no `db` field; `DatabaseConnection` lives only inside `*ServiceImpl`
- Call handlers as plain async functions: `super::handler(State(state), Path(id)).await`
- The mock uses `Mutex<HashMap<K, V>>` for interior mutability without `mut`
- For integration tests that need real DB behavior, test `service_impl` directly against a test database

## Registration Checklist

When adding a new domain, verify every item:

- [ ] Entity created in `entity/` crate
- [ ] Migration created in `migration/` crate
- [ ] Service trait defined in `service.rs`
- [ ] Service impl created in `service_impl.rs`
- [ ] `Arc<dyn <Domain>Service>` field added to `AppState`
- [ ] Field included in `Debug` impl for `AppState`
- [ ] Import added to `src/app.rs`
- [ ] Service constructed in `create_app()`
- [ ] Handler files created with `#[utoipa::path]` annotations
- [ ] `router()` function defined in `mod.rs`
- [ ] Route registered via `.nest("/api/v1/<domain>", <domain>_route)` in `create_app()`
- [ ] Mock service created in `service_mock.rs` under `#[cfg(test)]`
- [ ] Tests written inline in handler files

## Reference Files

| Purpose | Path |
|---------|------|
| App wiring & AppState | `src/app.rs` |
| Error types | `src/app/error.rs` |
| Full domain example | `src/features/users/` |
| Simple domain (no service) | `src/features/health.rs` |
| Config | `src/config.rs` |
| Entry point | `src/main.rs` |
