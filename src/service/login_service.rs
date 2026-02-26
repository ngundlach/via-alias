use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use async_trait::async_trait;
use jsonwebtoken::{Algorithm, EncodingKey, Header, jws::encode};
use uuid::Uuid;

use crate::{
    data::UserRepo,
    model::{User, UserClaimsDTO, UserCredentialsDTO, UserTokenDTO},
    service::{DbServiceError, LoginService},
};

pub struct LoginServiceImpl {
    repo: Arc<dyn UserRepo + Send + Sync>,
}
impl LoginServiceImpl {
    pub fn new(user_repo: Arc<dyn UserRepo + Send + Sync>) -> Self {
        LoginServiceImpl { repo: user_repo }
    }
    fn expiration_time(dur: Duration) -> usize {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is before Unix epoch")
            .checked_add(dur)
            .expect("timestamp overflow")
            .as_secs() as usize
    }

    fn create_token(&self, user: &User, jwt_secret: &str) -> Result<UserTokenDTO, DbServiceError> {
        let expiration_time = Self::expiration_time(Duration::from_mins(15));
        let user_claims = UserClaimsDTO {
            user_id: user.id.clone(),
            is_admin: user.is_admin,
            exp: expiration_time,
            jti: Uuid::new_v4().to_string(),
        };

        let header = Header::new(Algorithm::HS512);
        let token = encode(
            &header,
            Some(&user_claims),
            &EncodingKey::from_secret(jwt_secret.as_bytes()),
        )
        .map_err(|_| DbServiceError::LoginError("Error during token creation".to_owned()))?;

        let user_token = UserTokenDTO {
            access_token: format!("{}.{}.{}", token.protected, token.payload, token.signature),
            expires_in: expiration_time,
            refresh_token: Uuid::new_v4().to_string(),
            token_type: "Bearer".to_string(),
        };
        Ok(user_token)
    }
}

#[async_trait]
impl LoginService for LoginServiceImpl {
    async fn check_user_credentials(
        &self,
        user: &UserCredentialsDTO,
        user_data: &User,
    ) -> Result<(), DbServiceError> {
        let argon2 = Argon2::default();
        let parsed_hash = PasswordHash::new(&user_data.pwhash)
            .map_err(|e| DbServiceError::DatabaseError(e.to_string()))?;
        argon2
            .verify_password(user.pw.as_bytes(), &parsed_hash)
            .map_err(|e| DbServiceError::WrongCredentials(e.to_string()))?;
        Ok(())
    }

    async fn get_user_data(&self, user: &UserCredentialsDTO) -> Result<User, DbServiceError> {
        let user_data = self
            .repo
            .read_user_by_name(&user.name)
            .await
            .map_err(DbServiceError::from)?;
        Ok(user_data)
    }

    async fn login_user(
        &self,
        user: &UserCredentialsDTO,
        jwt_secret: &str,
    ) -> Result<UserTokenDTO, DbServiceError> {
        let user_data = self.get_user_data(user).await?;
        self.check_user_credentials(user, &user_data).await?;
        let jwt = self.create_token(&user_data, jwt_secret)?;
        Ok(jwt)
    }
}
