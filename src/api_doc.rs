use axum::Router;
use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_swagger_ui::SwaggerUi;

use crate::model::{
    FullRedirectListDTO, PasswordChangeDataDTO, Redirect, RedirectDTO, RedirectListDTO,
    UpdateUrlDTO, UserCredentialsDTO, UserRegistrationDTO, UserRegistrationTokenDTO, UserTokenDTO,
};
use crate::{controller::admin, model::UserDTO};
use crate::{controller::login, model::DeletedUserDTO};
use crate::{controller::redirect, model::DeletedUserResourceDTO};
use crate::{controller::user, model::UserListDTO};
use crate::{health_check, model::SimpleUserDTO};

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Auth", description = "Endpoints that handle user authentication."),
        (name = "Redirects", description = "These endpoints handle redirect management."),
        (name = "Users", description = "These endpoints handle user management."),
        (name = "Admin", description = "Admin management endpoints. All these endpoints require a valid JWT with admin claims."),
    ),
    info(title = "Via-Alias API",
        license(name = "MIT",
            identifier = "MIT"
        ),
        description = "API for [Via-Alias](https://github.com/ngundlach/via-alias). This API can be used to create and manage redirects as well as users.
        <br><br>**Authentication:**<br>
        Most Endpoints require a Bearer token obtained from `POST /api/auth/login`.
        Include it in the `Authorization` header as `Bearer <token>`."
    ),
    servers((url = "https://example.com", description = "Example:")),
    paths(
        login::login_user_handler,
        admin::request_user_registration_token_handler,
        admin::get_all_redirects_admin_handler,
        admin::delete_redirect_admin_handler,
        admin::user_info_admin_handler,
        admin::all_users_info_admin_handler,
        admin::delete_user_admin_handler,
        user::register_user_handler,
        user::simple_user_info_handler,
        user::change_user_password_handler,
        redirect::create_redirect_handler,
        redirect::get_all_user_redirects_handler,
        redirect::update_redirect_handler,
        redirect::delete_redirect_handler,
        redirect::follow_redirect_handler,
        health_check::health_check_handler,
    ),
    components(schemas(
        UserDTO, DeletedUserDTO, DeletedUserResourceDTO, SimpleUserDTO, UserListDTO, UserCredentialsDTO,
        PasswordChangeDataDTO, UserTokenDTO, UserRegistrationTokenDTO, UserRegistrationDTO,
        Redirect, FullRedirectListDTO, RedirectDTO, RedirectListDTO, UpdateUrlDTO
    )),
    modifiers(&SecurityAddon)
)]
pub(crate) struct ApiDoc;
struct SecurityAddon;
impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}
pub fn api_doc_router() -> Router {
    let ui = SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi());
    Router::new().merge(ui)
}
