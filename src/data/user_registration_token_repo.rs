use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    data::UserRegistrationTokenRepo, model::UserRegistrationToken, service::DbServiceError,
};

pub(crate) struct UserRegistrationTokenInMemoryImpl {
    user_registration_tokens: Arc<RwLock<HashMap<String, UserRegistrationToken>>>,
}

impl UserRegistrationTokenInMemoryImpl {
    pub(crate) fn new() -> UserRegistrationTokenInMemoryImpl {
        UserRegistrationTokenInMemoryImpl {
            user_registration_tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl UserRegistrationTokenRepo for UserRegistrationTokenInMemoryImpl {
    async fn create_user_registration_token(
        &self,
    ) -> Result<UserRegistrationToken, DbServiceError> {
        let exp_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("")
            .checked_add(Duration::from_mins(30))
            .expect("");
        let new_user_token = UserRegistrationToken {
            registration_token: Uuid::new_v4().to_string(),
            exp_time,
        };
        let mut guard = self.user_registration_tokens.write().await;
        guard.insert(
            new_user_token.registration_token.clone(),
            new_user_token.clone(),
        );

        Ok(new_user_token)
    }

    async fn read_token(&self, token_str: &str) -> Result<UserRegistrationToken, DbServiceError> {
        let guard = self.user_registration_tokens.read().await;

        let token = guard.get(token_str).ok_or(DbServiceError::NotFoundError)?;
        Ok(token.clone())
    }

    async fn delete_user_registration_token(
        &self,
        token: &UserRegistrationToken,
    ) -> Result<(), DbServiceError> {
        let mut guard = self.user_registration_tokens.write().await;
        guard
            .remove_entry(&token.registration_token)
            .map(|_| ())
            .ok_or(DbServiceError::DatabaseError(
                "error deleting registration token".to_owned(),
            ))
    }
}
