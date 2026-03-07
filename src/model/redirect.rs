use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[derive(Debug, Clone, Deserialize, Serialize, sqlx::FromRow, PartialEq)]
pub(crate) struct Redirect {
    pub id: String,
    pub alias: String,
    pub url: String,
    pub owner: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FullRedirectListDTO {
    pub redirects: Vec<Redirect>,
}

#[derive(Deserialize, Serialize, sqlx::FromRow, Debug, Clone, PartialEq, ToSchema)]
#[schema(title = "Redirect")]
pub(crate) struct RedirectDTO {
    #[schema(example = "gh")]
    pub alias: String,
    #[schema(example = "http://www.github.com")]
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

#[derive(Deserialize)]
pub(crate) struct UpdateUrlDTO {
    pub url: String,
}
