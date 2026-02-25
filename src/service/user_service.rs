use std::sync::Arc;

use argon2::{
    Argon2,
    password_hash::{Error, PasswordHasher, SaltString, rand_core::OsRng},
};
use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    data::UserRepo,
    model::{User, UserCredentialsDTO, UserDTO},
    service::{DbServiceError, UserService},
};

pub struct UserServiceImpl {
    repo: Arc<dyn UserRepo + Send + Sync>,
}
impl UserServiceImpl {
    pub fn new(user_repo: Arc<dyn UserRepo + Send + Sync>) -> Self {
        UserServiceImpl { repo: user_repo }
    }

    fn create_password_hash_string(user_password: &str) -> Result<String, Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2.hash_password(user_password.as_bytes(), &salt)?;
        Ok(hash.to_string())
    }
}
#[async_trait]
impl UserService for UserServiceImpl {
    async fn register_user(&self, user: &UserCredentialsDTO) -> Result<UserDTO, DbServiceError> {
        let uuid = Uuid::new_v4();
        let hash = UserServiceImpl::create_password_hash_string(&user.pw)
            .map_err(|e| DbServiceError::DatabaseError(e.to_string()))?;

        let new_user = User {
            id: uuid.to_string(),
            name: user.name.clone(),
            pwhash: hash.to_string(),
            is_admin: false,
        };

        self.repo
            .create_user(&new_user)
            .await
            .map_err(DbServiceError::from)
    }

    async fn get_admin_count(&self) -> Result<i64, DbServiceError> {
        self.repo
            .count_user_with_is_admin()
            .await
            .map_err(DbServiceError::from)
    }

    async fn update_user(&self, user: &UserDTO) -> Result<(), DbServiceError> {
        let res = self
            .repo
            .update_user(user)
            .await
            .map_err(DbServiceError::from)?;
        if res < 1 {
            return Err(DbServiceError::NotFoundError);
        }
        Ok(())
    }
}
