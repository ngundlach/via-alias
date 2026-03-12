use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get},
};

use crate::{
    AppContext, middleware,
    model::{DeletedUserDTO, FullRedirectListDTO, UserDTO, UserListDTO, UserRegistrationTokenDTO},
    service::DbServiceError,
};

pub(crate) fn router() -> Router<AppContext> {
    Router::new()
        .route(
            "/api/admin/reg_token",
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

#[utoipa::path(get, 
    path = "/api/admin/reg_token", 
    tag="admin", 
    description = "Request a user registration token. Requires authentication. Pass a JWT as a bearer token in the Authorization header.",
    security(("bearer_auth" = [])),
    operation_id="reg_token_request", 
    responses(
        (status = StatusCode::OK, description = "Success. Returns valid registration token", body = UserRegistrationTokenDTO),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized. No valid access token."),
        (status = StatusCode::FORBIDDEN, description = "Forbidden. User authorized but doesn't have permission.")
))]
async fn request_user_registration_token_handler(
    State(app_context): State<AppContext>,
) -> impl IntoResponse {
    let res = app_context
        .user_service
        .create_user_registration_token(&app_context.app_config)
        .await;
    match res {
        Ok(t) => (StatusCode::OK, Json(t)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response(),
    }
}

#[utoipa::path(get, 
    path = "/api/admin/redirects",
    tag="admin", 
    description = "Get a list of all currently created redirects. Requires authentication. Pass a JWT as a bearer token in the Authorization header.",
    security(("bearer_auth" = [])),
    operation_id="get_all_redirects", 
    responses(
        (status = StatusCode::OK, description = "Success. Returns a list of all currently created redirects", body = FullRedirectListDTO),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized. No valid access token."),
        (status = StatusCode::FORBIDDEN, description = "Forbidden. User authorized but doesn't have permission.")
))]
async fn get_all_redirects_admin_handler(
    State(app_context): State<AppContext>,
) -> impl IntoResponse {
    let res = app_context.redirect_service.get_all_redirects().await;
    match res {
        Ok(t) => (StatusCode::OK, Json(t)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response(),
    }
}

#[utoipa::path(delete, 
    path = "/api/admin/redirects/{id}",
    params(
        ("id" = String, Path, description = "The redirect id."),
    ),
    tag="admin", 
    description = "Delete a redirect via its id. Requires authentication. Pass a JWT as a bearer token in the Authorization header.",
    security(("bearer_auth" = [])),
    operation_id="delete_redirect_by_id",
    responses(
        (status = StatusCode::NO_CONTENT, description = "No Content. Deletes a redirect"),
        (status = StatusCode::NOT_FOUND, description = "Not Found. Redirect doesn't exist."),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized. No valid access token."),
        (status = StatusCode::FORBIDDEN, description = "Forbidden. User authorized but doesn't have permission.")
))]
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

#[utoipa::path(get, 
    path = "/api/admin/users/{id}",
    params(
        ("id" = String, Path, description = "The user id."),
    ),
    tag="admin", 
    description = "Get data about specific user. Requires authentication. Pass a JWT as a bearer token in the Authorization header.",
    security(("bearer_auth" = [])),
    operation_id="get_user_data_admin", 
    responses(
        (status = StatusCode::OK, description = "Ok. Returns persisted userdata.", body = UserDTO),
        (status = StatusCode::NOT_FOUND, description = "Not Found. User doesn't exist."),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized. No valid access token."),
        (status = StatusCode::FORBIDDEN, description = "Forbidden. User authorized but doesn't have permission.")
))]
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

#[utoipa::path(get, 
    path = "/api/admin/users",
    tag="admin", 
    description = "Get a list of all users. Requires authentication. Pass a JWT as a bearer token in the Authorization header.",
    security(("bearer_auth" = [])),
    operation_id="get_all_users_data_admin", 
    responses(
        (status = StatusCode::OK, description = "Ok. Returns list of persisted userdata.", body = UserListDTO),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized. No valid access token."),
        (status = StatusCode::FORBIDDEN, description = "Forbidden. User authorized but doesn't have permission.")
))]
async fn all_users_info_admin_handler(State(app_context): State<AppContext>) -> impl IntoResponse {
    let res = app_context.user_service.get_all_users_info().await;
    match res {
        Ok(user) => (StatusCode::OK, Json(user)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response(),
    }
}

#[utoipa::path(delete, 
    path = "/api/admin/users/{id}",
    params(
        ("id" = String, Path, description = "The user id."),
    ),
    tag="admin", 
    description = "Delete a user. This will also delete all of the users registered redirects. Requires authentication. Pass a JWT as a bearer token in the Authorization header.",
    security(("bearer_auth" = [])),
    operation_id="delete_user", 
    responses(
        (status = StatusCode::OK, description = "Ok. Returns userid and amount of deleted redirects", body = DeletedUserDTO),
        (status = StatusCode::NOT_FOUND, description = "Not Found. User doesn't exist."),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized. No valid access token."),
        (status = StatusCode::FORBIDDEN, description = "Forbidden. User authorized but doesn't have permission.")
))]
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
