use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use jsonwebtoken::{EncodingKey, Header, jws::encode};
use uuid::Uuid;

use crate::{
    JwtConfig,
    data::UserRepo,
    model::{User, UserClaimsDTO, UserCredentialsDTO, UserTokenDTO},
    service::{DbServiceError, LoginService, validator},
};

pub struct LoginServiceImpl {
    repo: Arc<dyn UserRepo + Send + Sync>,
}
impl LoginServiceImpl {
    pub fn new(user_repo: Arc<dyn UserRepo + Send + Sync>) -> Self {
        LoginServiceImpl { repo: user_repo }
    }
    fn expiration_time(dur: Duration) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is before Unix epoch")
            .checked_add(dur)
            .expect("timestamp overflow")
            .as_secs()
    }

    fn create_token(user: &User, jwt_config: &JwtConfig) -> Result<UserTokenDTO, DbServiceError> {
        let expiration_time = Self::expiration_time(Duration::from_secs(jwt_config.jwt_ttl));
        let user_claims = UserClaimsDTO {
            user_id: user.id.clone(),
            is_admin: user.is_admin,
            exp: expiration_time,
            jti: Uuid::new_v4().to_string(),
        };

        let header = Header::new(jwt_config.jwt_alg);
        let token = encode(
            &header,
            Some(&user_claims),
            &EncodingKey::from_secret(jwt_config.jwt_secret.as_bytes()),
        )
        .map_err(|_| DbServiceError::AuthError("Error during token creation".to_owned()))?;

        let user_token = UserTokenDTO {
            access_token: format!("{}.{}.{}", token.protected, token.payload, token.signature),
            expires_in: jwt_config.jwt_ttl,
            // refresh_token: Uuid::new_v4().to_string(),
            token_type: "Bearer".to_string(),
        };
        Ok(user_token)
    }

    async fn get_user_data(&self, user: &UserCredentialsDTO) -> Result<User, DbServiceError> {
        let user_data = self
            .repo
            .read_user_by_name(&user.name)
            .await
            .map_err(DbServiceError::from)?;
        Ok(user_data)
    }
}

#[async_trait]
impl LoginService for LoginServiceImpl {
    async fn login_user(
        &self,
        user: &UserCredentialsDTO,
        jwt_config: &JwtConfig,
    ) -> Result<UserTokenDTO, DbServiceError> {
        let user_data = self.get_user_data(user).await?;
        validator::check_user_credentials(user, &user_data)?;
        let jwt = Self::create_token(&user_data, jwt_config)?;
        Ok(jwt)
    }
}
