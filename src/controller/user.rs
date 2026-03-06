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
    service::{DbServiceError, ValidationErrorResponse},
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
) -> impl IntoResponse {
    let password_change = UserPasswordChangeDTO {
        user_id: user_claims.user_id,
        pw: passwords,
    };
    let res = app_context
        .user_service
        .change_user_pw(&password_change)
        .await;
    match res {
        Ok(()) => StatusCode::OK,
        Err(DbServiceError::NotFoundError) => StatusCode::NOT_FOUND,
        Err(DbServiceError::AuthError(_)) => StatusCode::UNAUTHORIZED,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn register_user_handler(
    State(app_content): State<AppContext>,
    Json(payload): Json<UserRegistrationDTO>,
) -> impl IntoResponse {
    let user_credentials = UserCredentialsDTO {
        name: payload.name,
        pw: payload.pw,
    };
    let res = app_content
        .user_service
        .register_user_with_token(&user_credentials, &payload.token)
        .await;
    match res {
        Ok(e) => (StatusCode::CREATED, Json(e)).into_response(),
        Err(DbServiceError::AuthError(_)) => StatusCode::UNAUTHORIZED.into_response(),
        Err(DbServiceError::PayloadValidationError(s, e)) => ValidationErrorResponse {
            on_item: s,
            errors: e,
        }
        .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn simple_user_info_handler(
    State(app_context): State<AppContext>,
    Extension(user_claims): Extension<UserClaimsDTO>,
) -> impl IntoResponse {
    let res = app_context
        .user_service
        .get_simple_user_info(&user_claims.user_id)
        .await;
    match res {
        Ok(user) => (StatusCode::OK, Json(user)).into_response(),
        Err(DbServiceError::AuthError(_)) => StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
