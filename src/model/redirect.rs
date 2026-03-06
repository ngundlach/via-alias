use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, Deserialize, Serialize, sqlx::FromRow, PartialEq)]
pub(crate) struct Redirect {
    pub id: String,
    pub alias: String,
    pub url: String,
    pub owner: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub(crate) struct FullRedirectListDTO {
    pub redirects: Vec<Redirect>,
}

#[derive(Default, Deserialize, Serialize, sqlx::FromRow, Debug, Clone, PartialEq)]
pub(crate) struct RedirectDTO {
    pub alias: String,
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

#[derive(Default, Deserialize, Serialize)]
pub(crate) struct RedirectCreationDTO {
    pub redirect: RedirectDTO,
    pub owner: String,
}

#[derive(Default, Deserialize, Serialize, sqlx::FromRow, Debug)]
pub(crate) struct RedirectListDTO {
    pub redirects: Vec<RedirectDTO>,
}

#[derive(Default, Deserialize, Serialize)]
pub(crate) struct UpdateUrlDTO {
    pub url: String,
}
