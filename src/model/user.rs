use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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

#[derive(Deserialize, ToSchema)]
#[schema(title = "UserCredentials")]
pub struct UserCredentialsDTO {
    #[schema(example = "luke")]
    pub name: String,
    #[schema(example = "superjedimeister1337")]
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

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[schema(title = "UserAccessToken")]
pub struct UserTokenDTO {
    #[schema(
        example = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzUxMiJ9.eyJ1c2VyX2lkIjoiMjE1ZDU3YzAtMDM1My00Nzc2LWFjNTYtZDBiZWY5YTUxZTJjIiwiaXNfYWRtaW4iOmZhbHNlLCJleHAiOjE3NzI4OTczOTMsImp0aSI6IjRiZTcyNTUwLTZlYTUtNGQ3NS1iM2Q4LTdiNzJiYTEwZjE1MiJ9.qwy7zLn611SkZzI5mFJPwtRGjvjD0xmprSoMUII7xjcxRPjAbTEKH9gFIewYRGdwtcg0I-EhnttYMNZkqlmZNQ"
    )]
    pub access_token: String,
    // pub refresh_token: String,
    #[schema(example = "Bearer")]
    pub token_type: String,
    #[schema(example = "900")]
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
