use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};

use crate::{
    AppContext,
    model::{UserCredentialsDTO, UserTokenDTO},
};

pub fn router() -> Router<AppContext> {
    Router::new().route("/api/auth/login", post(login_user_handler))
}

#[utoipa::path(post,
    path = "/api/auth/login",
    tag = "Auth",
    summary = "Login",
    description = "Authenticates a user with their credentials and returns a signed JWT access token on success.
    The token should be included in subsequent requests as a Bearer token in the `Authorization` header.",
    request_body = UserCredentialsDTO,
    security(),
    operation_id="login",
    responses(
        (status = StatusCode::OK, description = "User authenticated. Returns valid JWT access token.", body = UserTokenDTO),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized. Invalid username or password"),
))]
async fn login_user_handler(
    State(app_state): State<AppContext>,
    Json(user): Json<UserCredentialsDTO>,
) -> impl IntoResponse {
    let res = app_state
        .login_service
        .login_user(&user, &app_state.app_config.jwt_config)
        .await;
    match res {
        Ok(t) => (StatusCode::OK, Json(t)).into_response(),
        Err(_) => StatusCode::UNAUTHORIZED.into_response(),
    }
}
