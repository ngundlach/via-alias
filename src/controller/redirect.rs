use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};

use crate::{AppState, model::RedirectDTO};
use crate::{model::UpdateUrlDTO, service::DbServiceError};

#[derive(Serialize, Deserialize)]
struct ValidationErrorResponse {
    on_item: String,
    errors: Vec<String>,
}
impl IntoResponse for ValidationErrorResponse {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{alias}", get(get_redirect_handler))
        .route(
            "/api/redirects",
            post(create_redirect_handler).get(get_all_redirects_handler),
        )
        .route(
            "/api/redirects/{alias}",
            delete(delete_redirect_handler).patch(update_redirect_handler),
        )
}

async fn delete_redirect_handler(
    State(AppState { redirect_service }): State<AppState>,
    Path(alias): Path<String>,
) -> impl IntoResponse {
    let query = redirect_service.delete_redirect(&alias).await;
    match query {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(DbServiceError::NotFoundError) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
async fn get_all_redirects_handler(
    State(AppState { redirect_service }): State<AppState>,
) -> impl IntoResponse {
    let redirects = redirect_service.get_all_redirects().await;
    match redirects {
        Ok(found) => Json(found).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
async fn create_redirect_handler(
    State(AppState { redirect_service }): State<AppState>,
    Json(payload): Json<RedirectDTO>,
) -> impl IntoResponse {
    let query = redirect_service.create_redirect(&payload).await;
    match query {
        Ok(_) => (StatusCode::CREATED, Json(payload)).into_response(),
        Err(DbServiceError::PayloadValidationError(s, e)) => ValidationErrorResponse {
            on_item: s,
            errors: e,
        }
        .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn get_redirect_handler(
    State(AppState { redirect_service }): State<AppState>,
    Path(alias): Path<String>,
) -> impl IntoResponse {
    let redirect = redirect_service.get_redirect(&alias).await;
    match redirect {
        Ok(r) => Redirect::temporary(&r.url).into_response(),
        Err(DbServiceError::NotFoundError) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
async fn update_redirect_handler(
    State(AppState { redirect_service }): State<AppState>,
    Path(alias): Path<String>,
    Json(payload): Json<UpdateUrlDTO>,
) -> impl IntoResponse {
    let result = redirect_service.update_redirect(&alias, &payload).await;
    match result {
        Ok(_) => (
            StatusCode::OK,
            Json(RedirectDTO {
                alias,
                url: payload.url,
            }),
        )
            .into_response(),
        Err(DbServiceError::NotFoundError) => StatusCode::NOT_FOUND.into_response(),
        Err(DbServiceError::PayloadValidationError(s, e)) => ValidationErrorResponse {
            on_item: s,
            errors: e,
        }
        .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
