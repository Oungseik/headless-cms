/// Returns a static health check string to confirm the server is running.
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

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn handler_should_return_ok_with_message() {
        let result = super::handler().await;
        assert_eq!(result, "server is up and running");
    }
}
