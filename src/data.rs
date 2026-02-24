use async_trait::async_trait;

use crate::model::{RedirectDTO, UpdateUrlDTO};
mod redirect_repo;
pub use crate::data::redirect_repo::RedirectRepoSqliteImpl;

#[async_trait]
pub trait RedirectRepo: Send + Sync + 'static {
    async fn read_redirect_by_alias(&self, alias: &str) -> Result<RedirectDTO, sqlx::Error>;
    async fn create_redirect(&self, redirect: &RedirectDTO) -> Result<(), sqlx::Error>;
    async fn read_all_redirects(&self) -> Result<Vec<RedirectDTO>, sqlx::Error>;
    async fn delete_redirect_by_alias(&self, alias: &str) -> Result<u64, sqlx::Error>;
    async fn update_redirect_by_alias(
        &self,
        alias: &str,
        redirect: &UpdateUrlDTO,
    ) -> Result<u64, sqlx::Error>;
}
