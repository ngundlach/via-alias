use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get},
};

use crate::{AppContext, middleware, service::DbServiceError};

pub(crate) fn router() -> Router<AppContext> {
    Router::new()
        .route(
            "/api/admin/registrationtoken",
            get(request_user_registration_token_handler),
        )
        .route("/api/admin/redirects", get(get_all_redirects_admin_handler))
        .route(
            "/api/admin/redirects/{id}",
            delete(delete_redirect_admin_handler),
        )
        .route("/api/admin/users/{id}", get(user_info_admin_handler))
        .route("/api/admin/users/{id}", delete(delete_user_admin_handler))
        .route("/api/admin/users", get(all_users_info_admin_handler))
        .layer(axum::middleware::from_fn(middleware::is_admin_middleware))
}

async fn request_user_registration_token_handler(
    State(app_context): State<AppContext>,
) -> impl IntoResponse {
    let res = app_context
        .user_service
        .create_user_registration_token()
        .await;
    match res {
        Ok(t) => (StatusCode::OK, Json(t)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response(),
    }
}

async fn get_all_redirects_admin_handler(
    State(app_context): State<AppContext>,
) -> impl IntoResponse {
    let res = app_context.redirect_service.get_all_redirects().await;
    match res {
        Ok(t) => (StatusCode::OK, Json(t)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response(),
    }
}

async fn delete_redirect_admin_handler(
    State(app_context): State<AppContext>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let query = app_context
        .redirect_service
        .delete_redirect_by_id(&id)
        .await;
    match query {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(DbServiceError::NotFoundError) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response(),
    }
}

async fn user_info_admin_handler(
    State(app_context): State<AppContext>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    let res = app_context.user_service.get_user_info(&user_id).await;
    match res {
        Ok(user) => (StatusCode::OK, Json(user)).into_response(),
        Err(DbServiceError::NotFoundError) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response(),
    }
}

async fn all_users_info_admin_handler(State(app_context): State<AppContext>) -> impl IntoResponse {
    let res = app_context.user_service.get_all_users_info().await;
    match res {
        Ok(user) => (StatusCode::OK, Json(user)).into_response(),
        Err(DbServiceError::NotFoundError) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response(),
    }
}

async fn delete_user_admin_handler(
    State(app_context): State<AppContext>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    let res = app_context.user_service.delete_user(&user_id).await;
    match res {
        Ok(user) => (StatusCode::OK, Json(user)).into_response(),
        Err(DbServiceError::NotFoundError) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response(),
    }
}
