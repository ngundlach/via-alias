use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize, sqlx::FromRow, Debug, Clone, PartialEq)]
pub(crate) struct RedirectDTO {
    pub alias: String,
    pub url: String,
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
