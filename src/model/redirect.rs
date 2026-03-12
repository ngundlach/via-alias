use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[derive(Debug, Clone, Deserialize, Serialize, sqlx::FromRow, PartialEq, ToSchema)]
#[schema(title = "RedirectData")]
pub(crate) struct Redirect {
    #[schema(examples("ea07b388-0da5-4640-b30d-2f90467a612c"))]
    pub id: String,
    #[schema(examples("gh"))]
    pub alias: String,
    #[schema(examples("http://www.github.com"))]
    pub url: String,
    #[schema(examples("7484bf63-0c9a-41af-884e-e0fea7f0bb8e"))]
    pub owner: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[schema(title = "FullRedirectList")]
pub(crate) struct FullRedirectListDTO {
    pub redirects: Vec<Redirect>,
}

#[derive(Deserialize, Serialize, sqlx::FromRow, Debug, Clone, PartialEq, ToSchema)]
#[schema(title = "SimpleRedirectData")]
pub(crate) struct RedirectDTO {
    #[schema(examples("gh"))]
    pub alias: String,
    #[schema(examples("http://www.github.com"))]
    pub url: String,
}

impl From<Redirect> for RedirectDTO {
    fn from(value: Redirect) -> Self {
        Self {
            alias: value.alias,
            url: value.url,
        }
    }
}

pub(crate) struct RedirectCreationDTO {
    pub redirect: RedirectDTO,
    pub owner: String,
}

#[derive(Serialize, sqlx::FromRow, Debug, ToSchema)]
#[schema(title = "RedirectList")]
pub(crate) struct RedirectListDTO {
    pub redirects: Vec<RedirectDTO>,
}

#[derive(Deserialize, ToSchema)]
#[schema(title = "UpdateUrl")]
pub(crate) struct UpdateUrlDTO {
    #[schema(examples("http://my-new-redirect-url.de"))]
    pub url: String,
}
