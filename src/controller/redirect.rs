use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::{delete, get, patch, post},
};

use crate::{
    AppContext,
    model::{RedirectCreationDTO, RedirectDTO, RedirectListDTO, UserClaimsDTO},
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

#[utoipa::path(delete, 
    path = "/api/redirects/{alias}",
    tag="redirects", 
    security(("bearer_auth" = [])),
    description = "Delete redirect. Requires authentication. Pass a JWT as a bearer token in the Authorization header.",
    params(
        ("alias" = String, Path, description = "The redirect alias."),
    ),
    operation_id="delete_redirect", 
    responses(
        (status = StatusCode::NO_CONTENT, description = "No Content. Redirect deleted successfully."),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized. No valid access token."),
        (status = StatusCode::FORBIDDEN, description = "Forbidden. User authorized but doesn't have permission.")
))]
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
#[utoipa::path(get, 
    path = "/api/redirects",
    tag="redirects", 
    security(("bearer_auth" = [])),
    description = "Get a list of all redirects owned by the current user. Requires authentication. Pass a JWT as a bearer token in the Authorization header.",
    operation_id="get_user_redirects_list", 
    responses(
        (status = StatusCode::OK, description = "Ok. Returns list of redirects.", body = RedirectListDTO),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized. No valid access token."),
))]
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

#[utoipa::path(post, 
    path = "/api/redirects",
    tag="redirects", 
    description = "Create a new redirect. Requires authentication. Pass a JWT as a bearer token in the Authorization header.",
    security(("bearer_auth" = [])),
    request_body = RedirectDTO,
    operation_id="create_redirect", 
    responses(
        (status = StatusCode::CREATED, description = "Created. Returns list of redirects.", body = RedirectDTO),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized. No valid access token."),
        (status = StatusCode::BAD_REQUEST, description = "Alias or url don't match requirements."),
        (status = StatusCode::CONFLICT, description = "A redirect with that alias already exists.")
))]
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

#[utoipa::path(get, 
    path = "/{alias}",
    tag="redirects", 
    description = "Follow the redirect.",
    params(
        ("alias" = String, Path, description = "The redirect alias."),
    ),
    operation_id="follow_redirect", 
    responses(
        (status = StatusCode::NOT_FOUND, description = "Not Found. Redirect doesn't exist."), 
        (status = StatusCode::TEMPORARY_REDIRECT, description = "Temporary Redirect. Follow redirect",
            headers(
                ("Location" = String, description = "URL of the created resource")
            )
        ) 
    )
)]
pub(crate) async fn follow_redirect_handler(
    State(app_state): State<AppContext>,
    Path(alias): Path<String>,
) -> Result<impl IntoResponse, DbServiceError> {
    let redirect = app_state.redirect_service.get_redirect(&alias).await?;
    Ok(Redirect::temporary(&redirect.url).into_response())
}

#[utoipa::path(patch, 
    path = "/api/redirects/{alias}",
    tag="redirects", 
    description = "Create a new redirect. Requires authentication. Pass a JWT as a bearer token in the Authorization header.",
    params(
        ("alias" = String, Path, description = "The redirect alias."),
    ),
    security(("bearer_auth" = [])),
    operation_id="update_redirect", 
    responses(
        (status = StatusCode::OK, description = "Ok. Changed the url of the redirect.", body = RedirectDTO),
        (status = StatusCode::NOT_FOUND, description = "Not Found. Redirect doesn't exist."),
        (status = StatusCode::UNAUTHORIZED, description = "Unauthorized. No valid access token."),
        (status = StatusCode::FORBIDDEN, description = "Forbidden. User authenticated but doesn't have permission."),
))]
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
