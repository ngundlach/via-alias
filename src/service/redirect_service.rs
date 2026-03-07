use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    data::RedirectRepo,
    model::{
        FullRedirectListDTO, Redirect, RedirectCreationDTO, RedirectDTO, RedirectListDTO,
        UpdateUrlDTO,
    },
    service::{PayloadValidator, RedirectService, error::DbServiceError},
};

pub struct RedirectServiceImpl {
    repo: Arc<dyn RedirectRepo + Send + Sync>,
}
impl RedirectServiceImpl {
    pub(crate) fn new(repo: Arc<dyn RedirectRepo + Send + Sync>) -> Self {
        RedirectServiceImpl { repo }
    }
    fn validate_alias(alias: &str) -> Result<(), DbServiceError> {
        PayloadValidator::new(alias)
            .not_empty()
            .max_length(50)
            .valid_characters()
            .restricted("api")
            .restricted("healthcheck")
            .restricted("swagger-ui")
            .restricted("api-docs")
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
        let result = self
            .repo
            .read_redirect_by_alias(alias)
            .await
            .map_err(DbServiceError::from)?;
        Ok(result.into())
    }

    async fn create_redirect(&self, redirect: &RedirectCreationDTO) -> Result<(), DbServiceError> {
        RedirectServiceImpl::validate_alias(&redirect.redirect.alias)?;
        RedirectServiceImpl::validate_url(&redirect.redirect.url)?;
        let redirect = Redirect {
            id: Uuid::new_v4().to_string(),
            alias: redirect.redirect.alias.clone(),
            url: redirect.redirect.url.clone(),
            owner: redirect.owner.clone(),
        };

        self.repo
            .create_redirect(&redirect)
            .await
            .map_err(DbServiceError::from)
    }

    async fn get_all_redirects(&self) -> Result<FullRedirectListDTO, DbServiceError> {
        let redirects = self
            .repo
            .read_all_redirects()
            .await
            .map_err(DbServiceError::from)?;
        Ok(FullRedirectListDTO { redirects })
    }

    async fn get_all_user_redirects(
        &self,
        user_id: &str,
    ) -> Result<RedirectListDTO, DbServiceError> {
        self.repo
            .read_all_redirects_by_user_id(user_id)
            .await
            .map_err(DbServiceError::from)
            .map(|r| RedirectListDTO {
                redirects: r.into_iter().map(std::convert::Into::into).collect(),
            })
    }

    async fn delete_redirect_by_id(&self, id: &str) -> Result<(), DbServiceError> {
        let res = self
            .repo
            .delete_redirect_by_id(id)
            .await
            .map_err(DbServiceError::from)?;
        if res == 0 {
            return Err(DbServiceError::NotFoundError);
        }
        Ok(())
    }

    async fn delete_user_redirect(&self, alias: &str, user_id: &str) -> Result<(), DbServiceError> {
        let res = self
            .repo
            .delete_redirect_by_alias_with_user_id(alias, user_id)
            .await
            .map_err(DbServiceError::from)?;
        if res == 0 {
            self.repo.read_redirect_by_alias(alias).await?;
            return Err(DbServiceError::PermissionError(
                "User is not authorized to delete redirect".to_owned(),
            ));
        }
        Ok(())
    }

    async fn update_redirect(
        &self,
        alias: &str,
        redirect: &UpdateUrlDTO,
        user_id: &str,
    ) -> Result<(), DbServiceError> {
        RedirectServiceImpl::validate_url(&redirect.url)?;
        let res = self
            .repo
            .update_redirect_by_alias(alias, redirect, user_id)
            .await
            .map_err(DbServiceError::from)?;
        if res == 0 {
            self.repo.read_redirect_by_alias(alias).await?;
            return Err(DbServiceError::PermissionError(
                "User is not authorized to update redirect".to_owned(),
            ));
        }
        Ok(())
    }
}
