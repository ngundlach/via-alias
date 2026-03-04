use serde::{Deserialize, Serialize};

#[derive(Default, sqlx::FromRow, Debug, Clone, PartialEq)]
pub struct User {
    pub id: String,
    pub name: String,
    pub pwhash: String,
    pub is_admin: bool,
}

#[derive(Default, Deserialize, Serialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct PasswordChangeDataDTO {
    pub old_pw: String,
    pub new_pw: String,
}

#[derive(Debug)]
pub struct UserPasswordChangeDTO {
    pub user_id: String,
    pub pw: PasswordChangeDataDTO,
}

#[derive(Default, Deserialize, Serialize, Debug)]
pub struct UserTokenDTO {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UserClaimsDTO {
    pub user_id: String,
    pub is_admin: bool,
    pub exp: usize,
    pub jti: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Hash, Eq)]
pub struct UserRegistrationTokenDTO {
    pub registration_token: String,
}

#[derive(Default, Deserialize, Debug)]
pub struct UserRegistrationDTO {
    pub name: String,
    pub pw: String,
    pub token: String,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct UserRegistrationToken {
    pub registration_token: String,
    pub exp_at: u64,
}
