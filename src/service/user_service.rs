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
    data::{UserRegistrationTokenRepo, UserRepo},
    model::{User, UserCredentialsDTO, UserDTO, UserPasswordChangeDTO, UserRegistrationTokenDTO},
    service::{DbServiceError, UserService, validator},
};

pub struct UserServiceImpl {
    user_repo: Arc<dyn UserRepo + Send + Sync>,
    user_registration_token_repo: Arc<dyn UserRegistrationTokenRepo + Send + Sync>,
}

impl UserServiceImpl {
    pub fn new(
        user_repo: Arc<dyn UserRepo + Send + Sync>,
        user_registration_token_repo: Arc<dyn UserRegistrationTokenRepo + Send + Sync>,
    ) -> Self {
        UserServiceImpl {
            user_repo,
            user_registration_token_repo,
        }
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
            pwhash: hash,
            is_admin: false,
        };
        Ok(new_user)
    }
}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn register_user(
        &self,
        user: &UserCredentialsDTO,
        registration_token: &str,
    ) -> Result<UserDTO, DbServiceError> {
        let token = self
            .user_registration_token_repo
            .read_token(registration_token)
            .await
            .map_err(|_| DbServiceError::AuthError("token not found".to_owned()))?;

        if !token.is_valid() {
            _ = self
                .user_registration_token_repo
                .delete_user_registration_token(&token)
                .await;
            return Err(DbServiceError::AuthError("token is invalid".to_owned()));
        }

        let new_user =
            Self::create_user(user).map_err(|e| DbServiceError::DatabaseError(e.to_string()))?;

        let created_user = self.user_repo.create_user(&new_user).await?;

        self.user_registration_token_repo
            .delete_user_registration_token(&token)
            .await?;
        Ok(created_user)
    }

    async fn register_user_as_admin(
        &self,
        user: &UserCredentialsDTO,
    ) -> Result<UserDTO, DbServiceError> {
        let mut new_admin =
            Self::create_user(user).map_err(|e| DbServiceError::DatabaseError(e.to_string()))?;

        new_admin.is_admin = true;

        self.user_repo
            .create_user(&new_admin)
            .await
            .map_err(DbServiceError::from)
    }

    async fn get_admin_count(&self) -> Result<i64, DbServiceError> {
        self.user_repo
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
        }
        Ok(())
    }

    async fn update_user(&self, user: &UserDTO) -> Result<(), DbServiceError> {
        // let res = self
        //     .repo
        //     .update_user_by_id(&user)
        //     .await
        //     .map_err(DbServiceError::from)?;
        // if res < 1 {
        //     return Err(DbServiceError::NotFoundError);
        // }
        Ok(())
    }

    async fn change_user_pw(
        &self,
        password_change: &UserPasswordChangeDTO,
    ) -> Result<(), DbServiceError> {
        let user_data = self
            .user_repo
            .read_user_by_id(&password_change.user_id)
            .await?;

        validator::check_user_credentials(
            &UserCredentialsDTO {
                name: user_data.name.clone(),
                pw: password_change.pw.old_pw.clone(),
            },
            &user_data,
        )?;

        let mut user_data = user_data;
        user_data.pwhash = Self::create_password_hash_string(&password_change.pw.new_pw)
            .map_err(|e| DbServiceError::DatabaseError(e.to_string()))?;

        let changed_user = self.user_repo.update_user_by_id(&user_data).await?;

        if changed_user < 1 {
            return Err(DbServiceError::DatabaseError("".to_owned()));
        }
        Ok(())
    }

    async fn create_user_registration_token(
        &self,
    ) -> Result<UserRegistrationTokenDTO, DbServiceError> {
        let token = self
            .user_registration_token_repo
            .create_user_registration_token()
            .await?;
        Ok(UserRegistrationTokenDTO {
            registration_token: token.registration_token,
        })
    }
}
