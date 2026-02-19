pub mod redirect_controller {
    use axum::{
        Json, Router,
        extract::{Path, State},
        http::StatusCode,
        response::{IntoResponse, Redirect},
        routing::{get, patch, post},
    };

    use crate::{AppState, model::RedirectDTO};
    use crate::{data::DbServiceError, model::UpdateUrlDTO};

    pub fn router() -> Router<AppState> {
        Router::new()
            .route("/{alias}", get(get_redirect_handler))
            .route("/", post(create_redirect_handler))
            .route("/all", get(get_all_redirects_handler))
            .route("/{alias}", axum::routing::delete(delete_redirect_handler))
            .route("/{alias}", patch(update_redirect_handler))
    }

    async fn delete_redirect_handler(
        State(AppState { db }): State<AppState>,
        Path(alias): Path<String>,
    ) -> impl IntoResponse {
        let query = db.delete_redirect(&alias).await;
        match query {
            Ok(_) => StatusCode::NO_CONTENT,
            Err(DbServiceError::NotFoundError) => StatusCode::NOT_FOUND,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    async fn get_all_redirects_handler(
        State(AppState { db }): State<AppState>,
    ) -> impl IntoResponse {
        let redirects = db.read_all_redirects().await;
        match redirects {
            Ok(found) => Json(found).into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
    async fn create_redirect_handler(
        State(AppState { db }): State<AppState>,
        Json(payload): Json<RedirectDTO>,
    ) -> impl IntoResponse {
        let query = db.create_redirect(&payload).await;
        match query {
            Ok(_) => (StatusCode::OK, Json(payload)).into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }

    async fn get_redirect_handler(
        State(AppState { db }): State<AppState>,
        Path(alias): Path<String>,
    ) -> impl IntoResponse {
        let redirect = db.read_redirect_by_alias(&alias).await;
        match redirect {
            Ok(r) => Redirect::temporary(&r.url).into_response(),
            Err(DbServiceError::NotFoundError) => StatusCode::NOT_FOUND.into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
    async fn update_redirect_handler(
        State(AppState { db }): State<AppState>,
        Path(alias): Path<String>,
        Json(payload): Json<UpdateUrlDTO>,
    ) -> impl IntoResponse {
        let result = db.update_redirect(&alias, &payload).await;
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
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
