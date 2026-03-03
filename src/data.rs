use async_trait::async_trait;

use crate::{
    model::{RedirectCreationDTO, RedirectDTO, UpdateUrlDTO, User, UserDTO, UserRegistrationToken},
    service::DbServiceError,
};
mod redirect_repo;
mod user_registration_token_repo;
mod user_repo;
pub use crate::data::redirect_repo::RedirectRepoSqliteImpl;
pub(crate) use crate::data::user_registration_token_repo::UserRegistrationTokenInMemoryImpl;
pub use crate::data::user_repo::UserRepoSqliteImpl;

#[async_trait]
pub trait RedirectRepo: Send + Sync + 'static {
    async fn read_redirect_by_alias(&self, alias: &str) -> Result<RedirectDTO, sqlx::Error>;
    async fn create_redirect(&self, redirect: &RedirectCreationDTO) -> Result<(), sqlx::Error>;
    async fn read_all_redirects(&self) -> Result<Vec<RedirectDTO>, sqlx::Error>;
    async fn read_all_redirects_by_user_id(
        &self,
        user_id: &str,
    ) -> Result<Vec<RedirectDTO>, sqlx::Error>;
    async fn delete_redirect_by_alias(&self, alias: &str) -> Result<u64, sqlx::Error>;
    async fn delete_redirect_by_alias_with_user_id(
        &self,
        alias: &str,
        user_id: &str,
    ) -> Result<u64, sqlx::Error>;
    async fn update_redirect_by_alias(
        &self,
        alias: &str,
        redirect: &UpdateUrlDTO,
        user_id: &str,
    ) -> Result<u64, sqlx::Error>;
}

#[async_trait]
pub trait UserRepo: Send + Sync + 'static {
    async fn read_user_by_name(&self, name: &str) -> Result<User, sqlx::Error>;
    async fn read_user_by_id(&self, id: &str) -> Result<User, sqlx::Error>;
    async fn create_user(&self, user: &User) -> Result<UserDTO, sqlx::Error>;
    async fn count_user_with_is_admin(&self) -> Result<i64, sqlx::Error>;
    async fn update_user_by_id(&self, user: &User) -> Result<u64, sqlx::Error>;
}

#[async_trait]
pub(crate) trait UserRegistrationTokenRepo: Send + Sync + 'static {
    async fn create_user_registration_token(&self)
    -> Result<UserRegistrationToken, DbServiceError>;
    async fn read_token(&self, token: &str) -> Result<UserRegistrationToken, DbServiceError>;
    async fn delete_user_registration_token(
        &self,
        token: &UserRegistrationToken,
    ) -> Result<(), DbServiceError>;
}
