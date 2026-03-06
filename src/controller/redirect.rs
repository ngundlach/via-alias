use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
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
) -> impl IntoResponse {
    let query = app_state
        .redirect_service
        .delete_user_redirect(&alias, &user_claims.user_id)
        .await;
    match query {
        Ok(()) => StatusCode::NO_CONTENT,
        Err(DbServiceError::NotFoundError) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
async fn get_all_user_redirects_handler(
    State(app_state): State<AppContext>,
    Extension(user_claims): Extension<UserClaimsDTO>,
) -> impl IntoResponse {
    let redirects = app_state
        .redirect_service
        .get_all_user_redirects(&user_claims.user_id)
        .await;
    match redirects {
        Ok(found) => Json(found).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn create_redirect_handler(
    State(app_state): State<AppContext>,
    Extension(user_claims): Extension<UserClaimsDTO>,
    Json(payload): Json<RedirectDTO>,
) -> impl IntoResponse {
    let redirect_creation = RedirectCreationDTO {
        redirect: payload.clone(),
        owner: user_claims.user_id,
    };
    let query = app_state
        .redirect_service
        .create_redirect(&redirect_creation)
        .await;
    match query {
        Ok(()) => (StatusCode::CREATED, Json(payload)).into_response(),
        Err(DbServiceError::PayloadValidationError(s, e)) => ValidationErrorResponse {
            on_item: s,
            errors: e,
        }
        .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub(crate) async fn get_redirect_handler(
    State(app_state): State<AppContext>,
    Path(alias): Path<String>,
) -> impl IntoResponse {
    let redirect = app_state.redirect_service.get_redirect(&alias).await;
    match redirect {
        Ok(r) => Redirect::temporary(&r.url).into_response(),
        Err(DbServiceError::AuthError(_)) => StatusCode::UNAUTHORIZED.into_response(),
        Err(DbServiceError::NotFoundError) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
async fn update_redirect_handler(
    State(app_state): State<AppContext>,
    Path(alias): Path<String>,
    Extension(user_claims): Extension<UserClaimsDTO>,
    Json(payload): Json<UpdateUrlDTO>,
) -> impl IntoResponse {
    let result = app_state
        .redirect_service
        .update_redirect(&alias, &payload, &user_claims.user_id)
        .await;
    match result {
        Ok(()) => (
            StatusCode::OK,
            Json(RedirectDTO {
                alias,
                url: payload.url,
            }),
        )
            .into_response(),
        Err(DbServiceError::NotFoundError) => StatusCode::NOT_FOUND.into_response(),
        Err(DbServiceError::AuthError(_)) => StatusCode::UNAUTHORIZED.into_response(),
        Err(DbServiceError::PayloadValidationError(s, e)) => ValidationErrorResponse {
            on_item: s,
            errors: e,
        }
        .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
