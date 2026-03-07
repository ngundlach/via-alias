use axum::Router;
use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_swagger_ui::SwaggerUi;

use crate::controller::admin;
use crate::controller::login;
use crate::controller::redirect;
use crate::controller::user;
use crate::model::{UserCredentialsDTO, UserRegistrationTokenDTO, UserTokenDTO};

#[derive(OpenApi)]
#[openapi(
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
    ),
    components(schemas(UserCredentialsDTO, UserTokenDTO, UserRegistrationTokenDTO)),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;
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
