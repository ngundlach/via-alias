use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    data::RedirectRepo,
    model::{RedirectDTO, RedirectListDTO, UpdateUrlDTO},
    service::{PayloadValidator, RedirectService, error::DbServiceError},
};

pub struct RedirectServiceImpl {
    repo: Arc<dyn RedirectRepo + Send + Sync>,
}
impl RedirectServiceImpl {
    pub fn new(repo: impl RedirectRepo) -> Self {
        RedirectServiceImpl {
            repo: Arc::new(repo),
        }
    }
    fn validate_alias(alias: &str) -> Result<(), DbServiceError> {
        PayloadValidator::new(alias)
            .not_empty()
            .max_length(50)
            .valid_characters()
            .validate()
            .map_err(|e| DbServiceError::PayloadValidationError("alias".to_string(), e))
    }

    fn validate_url(url: &str) -> Result<(), DbServiceError> {
        PayloadValidator::new(url)
            .not_empty()
            .max_length(2048)
            .has_url_schema()
            .validate()
            .map_err(|e| DbServiceError::PayloadValidationError("url".to_string(), e))
    }
}

#[async_trait]
impl RedirectService for RedirectServiceImpl {
    async fn get_redirect(&self, alias: &str) -> Result<RedirectDTO, DbServiceError> {
        self.repo
            .read_redirect_by_alias(alias)
            .await
            .map_err(DbServiceError::from)
    }

    async fn create_redirect(&self, redirect: &RedirectDTO) -> Result<(), DbServiceError> {
        RedirectServiceImpl::validate_alias(&redirect.alias)?;
        RedirectServiceImpl::validate_url(&redirect.url)?;
        self.repo
            .create_redirect(redirect)
            .await
            .map_err(DbServiceError::from)
    }

    async fn get_all_redirects(&self) -> Result<RedirectListDTO, DbServiceError> {
        self.repo
            .read_all_redirects()
            .await
            .map_err(DbServiceError::from)
            .map(|r| RedirectListDTO { redirects: r })
    }
    async fn delete_redirect(&self, alias: &str) -> Result<(), DbServiceError> {
        let res = self
            .repo
            .delete_redirect_by_alias(alias)
            .await
            .map_err(DbServiceError::from)?;
        if res == 0 {
            return Err(DbServiceError::NotFoundError);
        }
        Ok(())
    }

    async fn update_redirect(
        &self,
        alias: &str,
        redirect: &UpdateUrlDTO,
    ) -> Result<(), DbServiceError> {
        RedirectServiceImpl::validate_url(&redirect.url)?;
        let res = self
            .repo
            .update_redirect_by_alias(alias, redirect)
            .await
            .map_err(DbServiceError::from)?;
        if res == 0 {
            return Err(DbServiceError::NotFoundError);
        }
        Ok(())
    }
}
