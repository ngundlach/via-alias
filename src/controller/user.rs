use axum::{
    Extension, Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
};

use crate::{
    AppContext,
    model::{
        PasswordChangeDataDTO, UserClaimsDTO, UserCredentialsDTO, UserPasswordChangeDTO,
        UserRegistrationDTO,
    },
    service::DbServiceError,
};

pub(crate) fn protected_user_management_router() -> Router<AppContext> {
    Router::new()
        .route("/api/users/password", patch(change_user_password_handler))
        .route("/api/users/info", get(simple_user_info_handler))
}

pub(crate) fn user_router() -> Router<AppContext> {
    Router::new().route("/api/users/register", post(register_user_handler))
}

async fn change_user_password_handler(
    State(app_context): State<AppContext>,
    Extension(user_claims): Extension<UserClaimsDTO>,
    Json(passwords): Json<PasswordChangeDataDTO>,
) -> Result<impl IntoResponse, DbServiceError> {
    let password_change = UserPasswordChangeDTO {
        user_id: user_claims.user_id,
        pw: passwords,
    };
    app_context
        .user_service
        .change_user_pw(&password_change)
        .await?;
    Ok(StatusCode::OK.into_response())
}

async fn register_user_handler(
    State(app_context): State<AppContext>,
    Json(payload): Json<UserRegistrationDTO>,
) -> Result<impl IntoResponse, DbServiceError> {
    let user_credentials = UserCredentialsDTO {
        name: payload.name,
        pw: payload.pw,
    };
    let res = app_context
        .user_service
        .register_user_with_token(&user_credentials, &payload.token)
        .await?;
    Ok((StatusCode::CREATED, Json(res)).into_response())
}

async fn simple_user_info_handler(
    State(app_context): State<AppContext>,
    Extension(user_claims): Extension<UserClaimsDTO>,
) -> Result<impl IntoResponse, DbServiceError> {
    let res = app_context
        .user_service
        .get_simple_user_info(&user_claims.user_id)
        .await?;
    Ok((StatusCode::OK, Json(res)).into_response())
}
