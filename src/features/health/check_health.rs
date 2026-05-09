#[utoipa::path(
    get,
    path = "/",
    operation_id = "check_health",
    responses((status = 200, description = "server is up and running")),
    tag = "Health",
)]
#[tracing::instrument]
pub async fn handler() -> &'static str {
    "server is up and running"
}
