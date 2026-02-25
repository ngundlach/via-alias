mod error;
mod redirect_service;
mod user_service;
mod validator;
use async_trait::async_trait;

pub use crate::model::{RedirectDTO, RedirectListDTO, UpdateUrlDTO};
use crate::model::{UserCredentialsDTO, UserDTO};
pub use crate::service::error::DbServiceError;
pub use crate::service::redirect_service::RedirectServiceImpl;
pub use crate::service::user_service::UserServiceImpl;
pub use crate::service::validator::PayloadValidator;

#[async_trait]
pub trait RedirectService {
    async fn get_redirect(&self, alias: &str) -> Result<RedirectDTO, DbServiceError>;
    async fn create_redirect(&self, redirect: &RedirectDTO) -> Result<(), DbServiceError>;
    async fn get_all_redirects(&self) -> Result<RedirectListDTO, DbServiceError>;
    async fn delete_redirect(&self, alias: &str) -> Result<(), DbServiceError>;
    async fn update_redirect(
        &self,
        alias: &str,
        redirect: &UpdateUrlDTO,
    ) -> Result<(), DbServiceError>;
}

#[async_trait]
pub trait UserService {
    async fn register_user(&self, user: &UserCredentialsDTO) -> Result<UserDTO, DbServiceError>;
    async fn get_admin_count(&self) -> Result<i64, DbServiceError>;
    async fn update_user(&self, user: &UserDTO) -> Result<(), DbServiceError>;
}
