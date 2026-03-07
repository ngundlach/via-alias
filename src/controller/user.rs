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
        PasswordChangeDataDTO, SimpleUserDTO, UserClaimsDTO, UserCredentialsDTO, UserPasswordChangeDTO, UserRegistrationDTO
    },
    service::{DbServiceError},
};

pub(crate) fn protected_user_management_router() -> Router<AppContext> {
    Router::new()
        .route("/api/users/password", patch(change_user_password_handler))
        .route("/api/users/info", get(simple_user_info_handler))
}

pub(crate) fn user_router() -> Router<AppContext> {
    Router::new().route("/api/users/register", post(register_user_handler))
}

#[utoipa::path(patch, 
    path = "/api/users/password",
    tag="users",
    description = "Change user-password. Requires authentication. Pass a JWT as a bearer token in the Authorization header.",
    request_body = UserPasswordChangeDTO, 
    security(("bearer_auth" = [])),
    operation_id="change_password", 
    responses(
        (status = StatusCode::OK, description = "Ok. Password changed successfully."),
        (status = StatusCode::BAD_REQUEST, description = "Bad Request. Current password is invalid or wew password doesn't match requirements."),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized. No valid access token."),
))]
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

#[utoipa::path(post, 
    path = "/api/users/register",
    tag="users", 
    description = "Register a new user. Requires a generated user registration token.",
    request_body = UserRegistrationDTO,
    operation_id="register_user", 
    responses(
        (status = StatusCode::CREATED, description = "Created. User registered successfully.", body = SimpleUserDTO),
        (status = StatusCode::CONFLICT, description="Conflict. A user with that name already exists."),
        (status = StatusCode::BAD_REQUEST, description="Bad Request. Password or username don't match requirements."),
))]
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

#[utoipa::path(get, 
    path = "/api/users/info",
    tag="users", 
    description = "Get userdata about own useraccount. Requires authentication. Pass a JWT as a bearer token in the Authorization header.",
    security(("bearer_auth" = [])),
    operation_id="get_users_data", 
    responses(
        (status = StatusCode::OK, description = "Ok. Returns userdata.", body = SimpleUserDTO),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized. No valid access token."),
))]
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
