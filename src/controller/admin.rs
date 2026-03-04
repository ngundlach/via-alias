use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};

use crate::{AppContext, middleware};

pub(crate) fn router() -> Router<AppContext> {
    Router::new()
        .route(
            "/api/admin/registrationtoken",
            get(request_user_registration_token),
        )
        .layer(axum::middleware::from_fn(middleware::is_admin_middleware))
}

async fn request_user_registration_token(
    State(app_context): State<AppContext>,
) -> impl IntoResponse {
    let res = app_context
        .user_service
        .create_user_registration_token()
        .await;
    match res {
        Ok(t) => (StatusCode::OK, Json(t)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response(),
    }
}
