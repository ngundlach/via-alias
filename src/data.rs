use async_trait::async_trait;
pub use error::DbServiceError;

pub use crate::data::sqlite_service::SqliteService;
use crate::model::{RedirectObject, RedirectObjectList, UpdateUrlObject};
mod error;
mod sqlite_service;
#[async_trait]
pub trait RedirectRepo: Send + Sync + 'static {
    async fn read_redirect_by_alias(&self, alias: &str) -> Result<RedirectObject, DbServiceError>;
    async fn create_redirect(&self, redirect: &RedirectObject) -> Result<(), DbServiceError>;
    async fn read_all_redirects(&self) -> Result<RedirectObjectList, DbServiceError>;
    async fn delete_redirect(&self, alias: &str) -> Result<(), DbServiceError>;
    async fn update_redirect(
        &self,
        alias: &str,
        redirect: &UpdateUrlObject,
    ) -> Result<(), DbServiceError>;
}
