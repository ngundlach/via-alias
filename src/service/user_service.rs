use std::sync::Arc;

use argon2::{
    Argon2,
    password_hash::{
        Error, PasswordHasher, SaltString,
        rand_core::{OsRng, RngCore},
    },
};
use async_trait::async_trait;
use base64::{Engine, engine::general_purpose};
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
    fn create_user(user: &UserCredentialsDTO) -> Result<User, Error> {
        let uuid = Uuid::new_v4();
        let hash = UserServiceImpl::create_password_hash_string(&user.pw)?;

        let new_user = User {
            id: uuid.to_string(),
            name: user.name.clone(),
            pwhash: hash.to_string(),
            is_admin: false,
        };
        Ok(new_user)
    }
}
#[async_trait]
impl UserService for UserServiceImpl {
    async fn register_user(&self, user: &UserCredentialsDTO) -> Result<UserDTO, DbServiceError> {
        let new_user =
            Self::create_user(user).map_err(|e| DbServiceError::DatabaseError(e.to_string()))?;
        self.repo
            .create_user(&new_user)
            .await
            .map_err(DbServiceError::from)
    }

    async fn register_user_as_admin(
        &self,
        user: &UserCredentialsDTO,
    ) -> Result<UserDTO, DbServiceError> {
        let mut new_admin =
            Self::create_user(user).map_err(|e| DbServiceError::DatabaseError(e.to_string()))?;
        new_admin.is_admin = true;
        self.repo
            .create_user(&new_admin)
            .await
            .map_err(DbServiceError::from)
    }

    async fn get_admin_count(&self) -> Result<i64, DbServiceError> {
        self.repo
            .count_user_with_is_admin()
            .await
            .map_err(DbServiceError::from)
    }

    async fn create_admin_first_start(&self) -> Result<(), DbServiceError> {
        let count = self.get_admin_count().await?;
        if count < 1 {
            println!("Creating admin account...");
            let mut bytes = [0u8; 16];
            OsRng.fill_bytes(&mut bytes);
            let pw = general_purpose::STANDARD.encode(bytes);
            let initial_admin = UserCredentialsDTO {
                name: "admin".to_owned(),
                pw,
            };
            self.register_user_as_admin(&initial_admin).await?;
            println!("----------------------------------------");
            println!("{:<10} {}", "name:", initial_admin.name);
            println!("{:<10} {}", "password:", initial_admin.pw);
            println!("\n!!! Remember to change the password !!!");
            println!("----------------------------------------");
        };
        Ok(())
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
