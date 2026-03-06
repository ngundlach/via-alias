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
    data::{UserRegistrationTokenRepo, UserRepo, UserRepoError},
    model::{
        DeletedUserDTO, DeletedUserResourceDTO, SimpleUserDTO, User, UserCredentialsDTO, UserDTO,
        UserListDTO, UserPasswordChangeDTO, UserRegistrationTokenDTO,
    },
    service::{DbServiceError, PayloadValidator, UserService, validator},
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

    fn validate_user_name(user_name: &str) -> Result<(), DbServiceError> {
        PayloadValidator::new(user_name)
            .not_empty()
            .min_length(3)
            .max_length(15)
            .valid_characters()
            .validate()
            .map_err(|e| DbServiceError::PayloadValidationError("name".to_string(), e))
    }

    fn validate_password(pw: &str) -> Result<(), DbServiceError> {
        PayloadValidator::new(pw)
            .not_empty()
            .min_length(12)
            .max_length(100)
            .valid_characters()
            .one_alphabetic()
            .one_numeric()
            .validate()
            .map_err(|e| DbServiceError::PayloadValidationError("password".to_string(), e))
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
    async fn register_user(&self, user: &UserCredentialsDTO) -> Result<UserDTO, DbServiceError> {
        Self::validate_user_name(&user.name)?;
        Self::validate_password(&user.pw)?;
        let new_user =
            Self::create_user(user).map_err(|e| DbServiceError::DatabaseError(e.to_string()))?;

        let created_user = self.user_repo.create_user(&new_user).await?;
        Ok(created_user.into())
    }

    async fn register_user_with_token(
        &self,
        user: &UserCredentialsDTO,
        registration_token: &str,
    ) -> Result<SimpleUserDTO, DbServiceError> {
        let token = self
            .user_registration_token_repo
            .read_token(registration_token)
            .await?;

        let created_user = self.register_user(user).await?;

        self.user_registration_token_repo
            .delete_user_registration_token(&token.registration_token)
            .await?;
        Ok(created_user.into())
    }

    async fn register_user_as_admin(
        &self,
        user: &UserCredentialsDTO,
    ) -> Result<UserDTO, DbServiceError> {
        let mut new_admin =
            Self::create_user(user).map_err(|e| DbServiceError::DatabaseError(e.to_string()))?;

        new_admin.is_admin = true;

        let user = self
            .user_repo
            .create_user(&new_admin)
            .await
            .map_err(DbServiceError::from)?;
        Ok(user.into())
    }

    async fn get_user_info(&self, user_id: &str) -> Result<UserDTO, DbServiceError> {
        let user = self.user_repo.read_user_by_id(user_id).await?;
        Ok(user.into())
    }

    async fn get_simple_user_info(&self, user_id: &str) -> Result<SimpleUserDTO, DbServiceError> {
        let user = self.user_repo.read_user_by_id(user_id).await?;
        Ok(user.into())
    }

    async fn get_all_users_info(&self) -> Result<UserListDTO, DbServiceError> {
        self.user_repo
            .read_users()
            .await
            .map_err(DbServiceError::from)
            .map(|r| UserListDTO {
                users: r.into_iter().map(std::convert::Into::into).collect(),
            })
    }

    async fn delete_user(&self, user_id: &str) -> Result<DeletedUserDTO, DbServiceError> {
        let res = self
            .user_repo
            .delete_user_by_id(user_id)
            .await
            .map_err(|e| match e {
                UserRepoError::IsAdmin => {
                    DbServiceError::DatabaseError("User is admin".to_string())
                }
                UserRepoError::Db(e) => DbServiceError::DatabaseError(e.to_string()),
            })?;

        if res.affected_user_rows < 1 {
            return Err(DbServiceError::NotFoundError);
        }

        Ok(DeletedUserDTO {
            user_id: user_id.to_owned(),
            deleted: DeletedUserResourceDTO {
                redirects: res.affected_resources,
            },
        })
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

    async fn change_user_pw(
        &self,
        password_change: &UserPasswordChangeDTO,
    ) -> Result<(), DbServiceError> {
        let user_data = self
            .user_repo
            .read_user_by_id(&password_change.user_id)
            .await?;

        validator::validate_user_credentials(
            &UserCredentialsDTO {
                name: user_data.name.clone(),
                pw: password_change.pw.old_pw.clone(),
            },
            &user_data,
        )?;
        Self::validate_password(&password_change.pw.new_pw)?;
        let mut user_data = user_data;
        user_data.pwhash = Self::create_password_hash_string(&password_change.pw.new_pw)
            .map_err(|e| DbServiceError::DatabaseError(e.to_string()))?;

        let changed_user = self.user_repo.update_user_by_id(&user_data).await?;

        if changed_user < 1 {
            return Err(DbServiceError::NotFoundError);
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
