use axum::{
    Extension, Json, Router, debug_handler, extract::State, http::StatusCode,
    response::IntoResponse, routing::patch,
};

use crate::{
    AppContext,
    model::{PasswordChangeDataDTO, UserClaimsDTO, UserPasswordChangeDTO},
    service::DbServiceError,
};

pub(crate) fn user_management_router() -> Router<AppContext> {
    Router::new().route("/api/users/password", patch(change_user_password_handler))
}
#[debug_handler]
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
        Ok(_) => StatusCode::OK,
        Err(DbServiceError::NotFoundError) => StatusCode::NOT_FOUND,
        Err(DbServiceError::AuthError(_)) => StatusCode::UNAUTHORIZED,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
