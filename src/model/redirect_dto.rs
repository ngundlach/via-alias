use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize, sqlx::FromRow, Debug, Clone, PartialEq)]
pub struct RedirectDTO {
    pub alias: String,
    pub url: String,
}
#[derive(Default, Deserialize, Serialize, sqlx::FromRow, Debug)]
pub struct RedirectListDTO {
    pub redirects: Vec<RedirectDTO>,
}
#[derive(Default, Deserialize, Serialize)]
pub struct UpdateUrlDTO {
    pub url: String,
}
