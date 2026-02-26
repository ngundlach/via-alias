use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize, sqlx::FromRow, Debug, Clone, PartialEq)]
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

#[derive(Default, Deserialize, Serialize, Debug)]
pub struct UserCredentialsDTO {
    pub name: String,
    pub pw: String,
}

#[derive(Default, Deserialize, Serialize, Debug)]
pub struct UserTokenDTO {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserClaimsDTO {
    pub user_id: String,
    pub is_admin: bool,
    pub exp: usize,
    pub jti: String,
}
