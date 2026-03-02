use axum::{Router, http::StatusCode, response::IntoResponse, routing::post};

use crate::AppContext;

pub(crate) fn user_management_router() -> Router<AppContext> {
    Router::new().route("/api/users/password", post(change_user_password_handler))
}
async fn change_user_password_handler() -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED.into_response()
}
