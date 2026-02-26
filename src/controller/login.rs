use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};

use crate::{AppState, model::UserCredentialsDTO};

pub fn router() -> Router<AppState> {
    Router::new().route("/api/auth/login", post(login_user_handler))
}

async fn login_user_handler(
    State(app_state): State<AppState>,
    Json(user): Json<UserCredentialsDTO>,
) -> impl IntoResponse {
    let res = app_state
        .login_service
        .login_user(&user, &app_state.app_config.jwt_secret)
        .await;
    match res {
        Ok(t) => (StatusCode::OK, Json(t)).into_response(),
        Err(_) => StatusCode::UNAUTHORIZED.into_response(),
    }
}
