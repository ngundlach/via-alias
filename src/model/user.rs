use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize, sqlx::FromRow, Debug, Clone, PartialEq)]
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

#[derive(Default, Deserialize, Serialize, Debug)]
pub struct UserRegistrationDTO {
    pub name: String,
    pub pw: String,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Hash, Eq)]
pub struct UserRegistrationToken {
    pub registration_token: String,
    pub exp_at: u64,
}
impl UserRegistrationToken {
    pub(crate) fn is_valid(&self) -> bool {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is before Unix epoch")
            .as_secs();
        self.exp_at > current_time
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use uuid::Uuid;

    use crate::model::UserRegistrationToken;

    #[test]
    fn test_token_is_valid() {
        let exp_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .checked_add(Duration::from_hours(2))
            .unwrap()
            .as_secs();
        let token = UserRegistrationToken {
            registration_token: Uuid::new_v4().to_string(),
            exp_at,
        };
        assert!(token.is_valid());
    }
}
