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
#[derive(Serialize, Debug, ToSchema)]
#[schema(title = "UserData")]
pub struct UserDTO {
    #[schema(example = "d64bcaad-8d86-48d2-b1f3-f1c03ac30fa3")]
    pub id: String,
    #[schema(example = "luke")]
    pub name: String,
    #[schema(example = "false")]
    pub is_admin: bool,
}

#[derive(Serialize, ToSchema)]
#[schema(title = "DeletedUser")]
pub struct DeletedUserDTO {
    #[schema(example = "d64bcaad-8d86-48d2-b1f3-f1c03ac30fa3")]
    pub user_id: String,
    pub deleted: DeletedUserResourceDTO,
}

#[derive(Serialize, ToSchema)]
#[schema(title = "DeletedUserResource")]
pub struct DeletedUserResourceDTO {
    #[schema(example = 5)]
    pub redirects: u64,
}
#[derive(Serialize, ToSchema)]
#[schema(title = "UserList")]
pub struct UserListDTO {
    pub users: Vec<UserDTO>,
}

#[derive(Serialize, ToSchema)]
#[schema(title = "SimpleUserData")]
pub struct SimpleUserDTO {
    #[schema(example = "d64bcaad-8d86-48d2-b1f3-f1c03ac30fa3")]
    pub id: String,
    #[schema(example = "luke")]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Hash, Eq, ToSchema)]
#[schema(title = "UserRegistrationToken")]
pub struct UserRegistrationTokenDTO {
    #[schema(example = "85e83a5a-4f49-4ea7-9df9-93c2c2cc9b8f")]
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
