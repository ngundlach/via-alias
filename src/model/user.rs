use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize, sqlx::FromRow, Debug)]
pub struct User {
    pub id: String,
    pub name: String,
    pub pwhash: String,
    pub is_admin: bool,
}

#[derive(Default, Deserialize, Serialize, sqlx::FromRow, Debug, Clone)]
pub struct UserDTO {
    pub id: String,
    pub name: String,
    pub is_admin: bool,
}
