use async_trait::async_trait;
use sqlx::{Pool, Sqlite};

use super::{RedirectRepo, error::DbServiceError};
use crate::model::{RedirectDTO, RedirectListDTO, UpdateUrlDTO};
#[derive(Clone)]
pub struct SqliteService {
    db: Pool<Sqlite>,
}
impl SqliteService {
    pub fn new(dbpool: Pool<Sqlite>) -> Self {
        SqliteService { db: dbpool }
    }
}
#[async_trait]
impl RedirectRepo for SqliteService {
    async fn read_redirect_by_alias(&self, alias: &str) -> Result<RedirectDTO, DbServiceError> {
        sqlx::query_as::<_, RedirectDTO>("SELECT alias, url FROM redirects WHERE alias = $1;")
            .bind(alias)
            .fetch_one(&self.db)
            .await
            .map_err(DbServiceError::from)
    }

    async fn create_redirect(&self, redirect: &RedirectDTO) -> Result<(), DbServiceError> {
        sqlx::query("INSERT INTO redirects (alias, url) VALUES ($1, $2);")
            .bind(&redirect.alias)
            .bind(&redirect.url)
            .execute(&self.db)
            .await
            .map_err(DbServiceError::from)?; // Explicitly call from
        Ok(())
    }

    async fn read_all_redirects(&self) -> Result<RedirectListDTO, DbServiceError> {
        let redirects = sqlx::query_as::<_, RedirectDTO>("SELECT alias, url FROM redirects;")
            .fetch_all(&self.db)
            .await
            .map_err(DbServiceError::from)?;
        Ok(RedirectListDTO { redirects })
    }

    async fn delete_redirect(&self, alias: &str) -> Result<(), DbServiceError> {
        let result = sqlx::query("DELETE FROM redirects WHERE alias = $1;")
            .bind(alias)
            .execute(&self.db)
            .await
            .map_err(DbServiceError::from)?;
        if result.rows_affected() == 0 {
            return Err(DbServiceError::NotFoundError);
        }
        Ok(())
    }

    async fn update_redirect(
        &self,
        alias: &str,
        redirect: &UpdateUrlDTO,
    ) -> Result<(), DbServiceError> {
        let result = sqlx::query("UPDATE redirects SET url = $1 WHERE alias = $2")
            .bind(&redirect.url)
            .bind(alias)
            .execute(&self.db)
            .await
            .map_err(DbServiceError::from)?;
        if result.rows_affected() == 0 {
            return Err(DbServiceError::NotFoundError);
        }

        Ok(())
    }
}
