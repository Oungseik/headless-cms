use std::sync::Arc;

use utoipa_axum::router::OpenApiRouter;

use crate::app::AppState;

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
}
