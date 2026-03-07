use axum::Router;
use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_swagger_ui::SwaggerUi;

use crate::controller::login;
use crate::model::{UserCredentialsDTO, UserTokenDTO};

#[derive(OpenApi)]
#[openapi(
    paths(
        login::login_user_handler,
    ),
    components(schemas(UserCredentialsDTO, UserTokenDTO)),
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
