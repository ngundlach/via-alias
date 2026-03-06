use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Debug, PartialEq, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub pwhash: String,
    pub is_admin: bool,
}

impl From<User> for UserDTO {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            name: value.name,
            is_admin: value.is_admin,
        }
    }
}
#[derive(Serialize, Debug)]
pub struct UserDTO {
    pub id: String,
    pub name: String,
    pub is_admin: bool,
}

#[derive(Serialize)]
pub struct DeletedUserDTO {
    pub user_id: String,
    pub deleted: DeletedUserResourceDTO,
}

#[derive(Serialize)]
pub struct DeletedUserResourceDTO {
    pub redirects: u64,
}
#[derive(Serialize)]
pub struct UserListDTO {
    pub users: Vec<UserDTO>,
}

#[derive(Serialize)]
pub struct SimpleUserDTO {
    pub id: String,
    pub name: String,
}

impl From<User> for SimpleUserDTO {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            name: value.name,
        }
    }
}

impl From<UserDTO> for SimpleUserDTO {
    fn from(value: UserDTO) -> Self {
        Self {
            id: value.id,
            name: value.name,
        }
    }
}

#[derive(Deserialize)]
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

#[derive(Deserialize, Serialize, Debug)]
pub struct UserTokenDTO {
    pub access_token: String,
    // pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UserClaimsDTO {
    pub user_id: String,
    pub is_admin: bool,
    pub exp: u64,
    pub jti: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Hash, Eq)]
pub struct UserRegistrationTokenDTO {
    pub registration_token: String,
}

#[derive(Deserialize, Debug)]
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
