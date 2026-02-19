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

#[cfg(test)]
mod tests {
    use crate::data::RedirectRepo;
    use crate::data::SqliteService;
    use crate::model::RedirectDTO;
    use crate::model::UpdateUrlDTO;
    use sqlx::sqlite::SqlitePool;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool
    }

    async fn seed_test_db(pool: &SqlitePool) -> Vec<RedirectDTO> {
        let dtos = vec![
            RedirectDTO {
                alias: "somealias".to_string(),
                url: "https://someurl.com".to_string(),
            },
            RedirectDTO {
                alias: "secondalias".to_string(),
                url: "https://secondurl.com".to_string(),
            },
            RedirectDTO {
                alias: "thirdalias".to_string(),
                url: "https://thirdurl.com".to_string(),
            },
        ];

        for dto in &dtos {
            insert_into_test_db(dto.to_owned(), &pool).await;
        }
        dtos
    }

    async fn read_from_test_db(alias: &str, pool: &SqlitePool) -> Result<RedirectDTO, sqlx::Error> {
        sqlx::query_as::<_, RedirectDTO>("SELECT alias, url FROM redirects WHERE alias = $1;")
            .bind(alias)
            .fetch_one(pool)
            .await
    }

    async fn insert_into_test_db(dto: RedirectDTO, pool: &SqlitePool) {
        sqlx::query("INSERT INTO redirects (alias, url) VALUES ($1, $2);")
            .bind(dto.alias)
            .bind(dto.url)
            .execute(pool)
            .await
            .unwrap();
    }
    async fn read_all_from_test_db(pool: &SqlitePool) -> Vec<RedirectDTO> {
        sqlx::query_as::<_, RedirectDTO>("SELECT alias, url FROM redirects;")
            .fetch_all(pool)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_create_redirect_success() {
        let pool = setup_test_db().await;
        let service = SqliteService::new(pool.clone());

        let dto = RedirectDTO {
            alias: "somealias".to_string(),
            url: "https://someurl.com".to_string(),
        };

        let result = service.create_redirect(&dto).await;
        assert!(result.is_ok());

        let fetched = read_from_test_db(&dto.alias, &pool).await.unwrap();
        assert_eq!(fetched.alias, "somealias");

        assert_eq!(fetched.url, "https://someurl.com");
    }

    #[tokio::test]
    async fn test_create_redirect_duplicate_alias_fails() {
        let pool = setup_test_db().await;
        let service = SqliteService::new(pool.clone());

        let dtos = seed_test_db(&pool).await;

        let duplicate = RedirectDTO {
            alias: dtos[0].alias.clone(),
            url: dtos[0].url.clone(),
        };

        let result = service.create_redirect(&duplicate).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_redirect_success() {
        let pool = setup_test_db().await;
        let service = SqliteService::new(pool.clone());

        let alias_name = "somealias";
        seed_test_db(&pool).await;
        let update_dto = UpdateUrlDTO {
            url: "https://someotherurl.com".to_string(),
        };
        let result = service.update_redirect(alias_name, &update_dto).await;
        assert!(result.is_ok());

        let updated = read_from_test_db(alias_name, &pool).await.unwrap();

        assert_eq!(updated.alias, "somealias");
        assert_eq!(updated.url, "https://someotherurl.com");
    }

    #[tokio::test]
    async fn test_update_wrong_redirect_fails() {
        let pool = setup_test_db().await;
        let service = SqliteService::new(pool.clone());

        seed_test_db(&pool).await;
        let update_dto = UpdateUrlDTO {
            url: "https://someotherurl.com".to_string(),
        };
        let result = service.update_redirect("somewrongalias", &update_dto).await;
        assert!(result.is_err());
        let notfound = read_from_test_db("somewrongalias", &pool).await;
        assert!(notfound.is_err());
    }

    #[tokio::test]
    async fn test_read_redirect_by_alias_success() {
        let pool = setup_test_db().await;
        let service = SqliteService::new(pool.clone());

        seed_test_db(&pool).await;
        let result = service.read_redirect_by_alias("somealias").await;

        assert!(result.is_ok());
        let result_values = result.unwrap();
        assert_eq!(result_values.alias, "somealias");
        assert_eq!(result_values.url, "https://someurl.com");
    }

    #[tokio::test]
    async fn test_read_redirect_by_alias_not_found() {
        let pool = setup_test_db().await;
        let service = SqliteService::new(pool);

        let result = service.read_redirect_by_alias("somealias").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_read_all_redirects_should_return_empty_list() {
        let pool = setup_test_db().await;
        let service = SqliteService::new(pool);

        let result = service.read_all_redirects().await;

        assert!(result.is_ok());
        let result_list = result.unwrap();
        assert!(result_list.redirects.is_empty())
    }

    #[tokio::test]
    async fn test_read_all_redirects_should_return_list() {
        let pool = setup_test_db().await;
        let service = SqliteService::new(pool.clone());

        let dtos = seed_test_db(&pool).await;

        let result = service.read_all_redirects().await;

        assert!(result.is_ok());

        let result_list = result.unwrap();

        assert_eq!(result_list.redirects.len(), 3);

        for result_item in dtos {
            assert!(result_list.redirects.contains(&result_item));
        }
    }

    #[tokio::test]
    async fn test_delete_redirect_success() {
        let pool = setup_test_db().await;
        let service = SqliteService::new(pool.clone());

        let dtos = seed_test_db(&pool).await;

        let result = service.delete_redirect(&dtos[0].alias).await;

        assert!(result.is_ok());
        let updated_list = read_all_from_test_db(&pool).await;
        assert_eq!(updated_list.len(), 2);
        assert!(updated_list.contains(&dtos[1]));
        assert!(updated_list.contains(&dtos[2]));
        assert!(!updated_list.contains(&dtos[0]));
    }

    #[tokio::test]
    async fn test_delete_redirect_failed() {
        let pool = setup_test_db().await;
        let service = SqliteService::new(pool.clone());

        let dtos = seed_test_db(&pool).await;

        let result = service.delete_redirect("invalidalias").await;

        assert!(result.is_err());
        let updated_list = read_all_from_test_db(&pool).await;
        assert_eq!(updated_list.len(), 3);
        for dto in &dtos {
            assert!(updated_list.contains(dto));
        }
    }
}
