use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::{delete, get, patch, post},
};

use crate::{
    AppContext,
    model::{RedirectCreationDTO, RedirectDTO, UserClaimsDTO},
    service::ValidationErrorResponse,
};
use crate::{model::UpdateUrlDTO, service::DbServiceError};

impl IntoResponse for ValidationErrorResponse {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}

pub(crate) fn router() -> Router<AppContext> {
    Router::new()
        .route("/api/redirects", post(create_redirect_handler))
        .route("/api/redirects", get(get_all_user_redirects_handler))
        .route("/api/redirects/{alias}", patch(update_redirect_handler))
        .route("/api/redirects/{alias}", delete(delete_redirect_handler))
}

async fn delete_redirect_handler(
    State(app_state): State<AppContext>,
    Path(alias): Path<String>,
    Extension(user_claims): Extension<UserClaimsDTO>,
) -> Result<Response, DbServiceError> {
    app_state
        .redirect_service
        .delete_user_redirect(&alias, &user_claims.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}
async fn get_all_user_redirects_handler(
    State(app_state): State<AppContext>,
    Extension(user_claims): Extension<UserClaimsDTO>,
) -> Result<impl IntoResponse, DbServiceError> {
    let redirects = app_state
        .redirect_service
        .get_all_user_redirects(&user_claims.user_id)
        .await?;
    Ok(Json(redirects).into_response())
}

async fn create_redirect_handler(
    State(app_state): State<AppContext>,
    Extension(user_claims): Extension<UserClaimsDTO>,
    Json(payload): Json<RedirectDTO>,
) -> Result<impl IntoResponse, DbServiceError> {
    let redirect_creation = RedirectCreationDTO {
        redirect: payload.clone(),
        owner: user_claims.user_id,
    };
    app_state
        .redirect_service
        .create_redirect(&redirect_creation)
        .await?;
    Ok((StatusCode::CREATED, Json(payload)).into_response())
}

pub(crate) async fn get_redirect_handler(
    State(app_state): State<AppContext>,
    Path(alias): Path<String>,
) -> Result<impl IntoResponse, DbServiceError> {
    let redirect = app_state.redirect_service.get_redirect(&alias).await?;
    Ok(Redirect::temporary(&redirect.url).into_response())
}

async fn update_redirect_handler(
    State(app_state): State<AppContext>,
    Path(alias): Path<String>,
    Extension(user_claims): Extension<UserClaimsDTO>,
    Json(payload): Json<UpdateUrlDTO>,
) -> Result<impl IntoResponse, DbServiceError> {
    app_state
        .redirect_service
        .update_redirect(&alias, &payload, &user_claims.user_id)
        .await?;
    Ok((
        StatusCode::OK,
        Json(RedirectDTO {
            alias,
            url: payload.url,
        }),
    )
        .into_response())
}
