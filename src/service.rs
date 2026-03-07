mod error;
mod login_service;
mod redirect_service;
mod user_service;
mod validator;
use async_trait::async_trait;

use crate::model::{
    DeletedUserDTO, FullRedirectListDTO, RedirectCreationDTO, SimpleUserDTO, UserCredentialsDTO,
    UserDTO, UserListDTO, UserPasswordChangeDTO, UserRegistrationTokenDTO, UserTokenDTO,
};
pub(crate) use crate::model::{RedirectDTO, RedirectListDTO, UpdateUrlDTO};
pub use crate::service::error::*;
pub use crate::service::login_service::LoginServiceImpl;
pub use crate::service::redirect_service::RedirectServiceImpl;
pub use crate::service::user_service::UserServiceImpl;
pub use crate::service::validator::PayloadValidator;
pub(crate) use crate::service::validator::validate_registration_token;
use crate::{AppConfig, JwtConfig};

#[async_trait]
pub trait RedirectService {
    async fn get_redirect(&self, alias: &str) -> Result<RedirectDTO, DbServiceError>;
    async fn create_redirect(&self, redirect: &RedirectCreationDTO) -> Result<(), DbServiceError>;
    async fn get_all_redirects(&self) -> Result<FullRedirectListDTO, DbServiceError>;
    async fn get_all_user_redirects(
        &self,
        user_id: &str,
    ) -> Result<RedirectListDTO, DbServiceError>;
    async fn delete_redirect_by_id(&self, id: &str) -> Result<(), DbServiceError>;
    async fn delete_user_redirect(&self, alias: &str, user_id: &str) -> Result<(), DbServiceError>;
    async fn update_redirect(
        &self,
        alias: &str,
        redirect: &UpdateUrlDTO,
        user_id: &str,
    ) -> Result<(), DbServiceError>;
}

#[async_trait]
pub trait UserService {
    async fn register_user(&self, user: &UserCredentialsDTO) -> Result<UserDTO, DbServiceError>;
    async fn register_user_with_token(
        &self,
        user: &UserCredentialsDTO,
        registration_token: &str,
    ) -> Result<SimpleUserDTO, DbServiceError>;
    async fn register_user_as_admin(
        &self,
        user: &UserCredentialsDTO,
    ) -> Result<UserDTO, DbServiceError>;
    async fn get_admin_count(&self) -> Result<i64, DbServiceError>;
    async fn get_simple_user_info(&self, user_id: &str) -> Result<SimpleUserDTO, DbServiceError>;
    async fn get_user_info(&self, user_id: &str) -> Result<UserDTO, DbServiceError>;
    async fn get_all_users_info(&self) -> Result<UserListDTO, DbServiceError>;
    async fn delete_user(&self, user_id: &str) -> Result<DeletedUserDTO, DbServiceError>;
    async fn create_admin_first_start(&self) -> Result<(), DbServiceError>;
    async fn change_user_pw(
        &self,
        password_change: &UserPasswordChangeDTO,
    ) -> Result<(), DbServiceError>;
    async fn create_user_registration_token(
        &self,
        app_config: &AppConfig,
    ) -> Result<UserRegistrationTokenDTO, DbServiceError>;
}

#[async_trait]
pub trait LoginService {
    async fn login_user(
        &self,
        user: &UserCredentialsDTO,
        jwt_config: &JwtConfig,
    ) -> Result<UserTokenDTO, DbServiceError>;
}
