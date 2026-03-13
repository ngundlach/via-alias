use axum::{http::StatusCode, response::IntoResponse};

#[utoipa::path(get,
    path = "/healthcheck",
    tag="healthcheck",
    summary = "healtcheck",
    description = "Check if the service is running",
    operation_id="healthcheck",
    security(),
    responses(
        (status = StatusCode::OK, description = "Via-Alias is running"),
))]
pub(crate) async fn health_check_handler() -> impl IntoResponse {
    StatusCode::OK
}
