use axum::{http::StatusCode, response::IntoResponse};

pub(crate) async fn health_check_handler() -> impl IntoResponse {
    StatusCode::OK
}
