use axum::{http::StatusCode, response::IntoResponse};

#[utoipa::path(get,
    path = "/healthcheck",
    tag="healthcheck",
    operation_id="healthcheck",
    responses(
        (status = StatusCode::OK, description = "Via-Alias is running"),
))]
pub(crate) async fn health_check_handler() -> impl IntoResponse {
    StatusCode::OK
}
