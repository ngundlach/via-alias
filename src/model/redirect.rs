use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize, sqlx::FromRow, Debug, Clone)]
pub struct RedirectObject {
    pub alias: String,
    pub url: String,
}
#[derive(Default, Deserialize, Serialize, sqlx::FromRow, Debug)]
pub struct RedirectObjectList {
    pub redirects: Vec<RedirectObject>,
}
#[derive(Default, Deserialize, Serialize)]
pub struct UpdateUrlObject {
    pub url: String,
}
