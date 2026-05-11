//! Dashboard authentication endpoints (login, register, logout, token refresh).

use std::sync::Arc;

use utoipa_axum::router::OpenApiRouter;

use crate::app::AppState;

/// Returns the dashboard auth router.
pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
}
