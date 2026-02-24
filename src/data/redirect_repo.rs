use async_trait::async_trait;
use sqlx::{Pool, Sqlite};

use crate::{
    data::RedirectRepo,
    model::{RedirectDTO, UpdateUrlDTO},
};

pub struct RedirectRepoSqliteImpl {
    db: Pool<Sqlite>,
}

impl RedirectRepoSqliteImpl {
    pub fn new(db: Pool<Sqlite>) -> Self {
        RedirectRepoSqliteImpl { db }
    }
}

#[async_trait]
impl RedirectRepo for RedirectRepoSqliteImpl {
    async fn read_redirect_by_alias(&self, alias: &str) -> Result<RedirectDTO, sqlx::Error> {
        sqlx::query_as::<_, RedirectDTO>("SELECT alias, url FROM redirects WHERE alias = $1;")
            .bind(alias)
            .fetch_one(&self.db)
            .await
    }

    async fn create_redirect(&self, redirect: &RedirectDTO) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO redirects (alias, url) VALUES ($1, $2);")
            .bind(&redirect.alias)
            .bind(&redirect.url)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    async fn read_all_redirects(&self) -> Result<Vec<RedirectDTO>, sqlx::Error> {
        let redirects = sqlx::query_as::<_, RedirectDTO>("SELECT alias, url FROM redirects;")
            .fetch_all(&self.db)
            .await?;
        Ok(redirects)
    }

    async fn delete_redirect_by_alias(&self, alias: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM redirects WHERE alias = $1;")
            .bind(alias)
            .execute(&self.db)
            .await?;
        Ok(result.rows_affected())
    }

    async fn update_redirect_by_alias(
        &self,
        alias: &str,
        redirect: &UpdateUrlDTO,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("UPDATE redirects SET url = $1 WHERE alias = $2")
            .bind(&redirect.url)
            .bind(alias)
            .execute(&self.db)
            .await?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::SqlitePool;

    use crate::{
        data::{RedirectRepo, RedirectRepoSqliteImpl},
        model::{RedirectDTO, UpdateUrlDTO},
    };

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
            insert_into_test_db(dto.to_owned(), pool).await;
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
        let repo = RedirectRepoSqliteImpl::new(pool.clone());

        let dto = RedirectDTO {
            alias: "somealias".to_string(),
            url: "https://someurl.com".to_string(),
        };

        let result = repo.create_redirect(&dto).await;
        assert!(result.is_ok());
        let fetched = read_from_test_db(&dto.alias, &pool).await.unwrap();
        assert_eq!(fetched.alias, "somealias");

        assert_eq!(fetched.url, "https://someurl.com");
    }

    #[tokio::test]
    async fn test_create_redirect_duplicate_alias_fails() {
        let pool = setup_test_db().await;
        let repo = RedirectRepoSqliteImpl::new(pool.clone());

        let dtos = seed_test_db(&pool).await;

        let duplicate = RedirectDTO {
            alias: dtos[0].alias.clone(),
            url: dtos[0].url.clone(),
        };

        let result = repo.create_redirect(&duplicate).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(sqlx::Error::Database(_))));
    }

    #[tokio::test]
    async fn test_update_redirect_success() {
        let pool = setup_test_db().await;
        let repo = RedirectRepoSqliteImpl::new(pool.clone());

        let alias_name = "somealias";
        seed_test_db(&pool).await;
        let update_dto = UpdateUrlDTO {
            url: "https://someotherurl.com".to_string(),
        };
        let result = repo.update_redirect_by_alias(alias_name, &update_dto).await;
        assert!(result.is_ok());

        let updated = read_from_test_db(alias_name, &pool).await.unwrap();

        assert_eq!(updated.alias, "somealias");
        assert_eq!(updated.url, "https://someotherurl.com");
    }

    #[tokio::test]
    async fn test_update_wrong_redirect_leads_to_no_updates() {
        let pool = setup_test_db().await;
        let repo = RedirectRepoSqliteImpl::new(pool.clone());

        seed_test_db(&pool).await;
        let update_dto = UpdateUrlDTO {
            url: "https://someotherurl.com".to_string(),
        };
        let result = repo
            .update_redirect_by_alias("somewrongalias", &update_dto)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        let notfound = read_from_test_db("somewrongalias", &pool).await;
        assert!(notfound.is_err());
    }

    #[tokio::test]
    async fn test_read_redirect_by_alias_success() {
        let pool = setup_test_db().await;
        let repo = RedirectRepoSqliteImpl::new(pool.clone());

        seed_test_db(&pool).await;
        let result = repo.read_redirect_by_alias("somealias").await;

        assert!(result.is_ok());
        let result_values = result.unwrap();
        assert_eq!(result_values.alias, "somealias");
        assert_eq!(result_values.url, "https://someurl.com");
    }

    #[tokio::test]
    async fn test_read_redirect_by_alias_not_found() {
        let pool = setup_test_db().await;
        let repo = RedirectRepoSqliteImpl::new(pool.clone());

        let result = repo.read_redirect_by_alias("somealias").await;

        assert!(result.is_err());
        assert!(matches!(result, Err(sqlx::Error::RowNotFound)));
    }

    #[tokio::test]
    async fn test_read_all_redirects_should_return_empty_list() {
        let pool = setup_test_db().await;
        let repo = RedirectRepoSqliteImpl::new(pool.clone());

        let result = repo.read_all_redirects().await;

        assert!(result.is_ok());
        let result_list = result.unwrap();
        assert!(result_list.is_empty())
    }

    #[tokio::test]
    async fn test_read_all_redirects_should_return_list() {
        let pool = setup_test_db().await;
        let repo = RedirectRepoSqliteImpl::new(pool.clone());

        let dtos = seed_test_db(&pool).await;

        let result = repo.read_all_redirects().await;

        assert!(result.is_ok());

        let result_list = result.unwrap();

        assert_eq!(result_list.len(), 3);

        for result_item in dtos {
            assert!(result_list.contains(&result_item));
        }
    }

    #[tokio::test]
    async fn test_delete_redirect_success() {
        let pool = setup_test_db().await;
        let repo = RedirectRepoSqliteImpl::new(pool.clone());

        let dtos = seed_test_db(&pool).await;

        let result = repo.delete_redirect_by_alias(&dtos[0].alias).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
        let updated_list = read_all_from_test_db(&pool).await;
        assert_eq!(updated_list.len(), 2);
        assert!(updated_list.contains(&dtos[1]));
        assert!(updated_list.contains(&dtos[2]));
        assert!(!updated_list.contains(&dtos[0]));
    }

    #[tokio::test]
    async fn test_delete_unknown_redirect_leads_to_no_deletion() {
        let pool = setup_test_db().await;
        let repo = RedirectRepoSqliteImpl::new(pool.clone());

        let dtos = seed_test_db(&pool).await;

        let result = repo.delete_redirect_by_alias("invalidalias").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        let updated_list = read_all_from_test_db(&pool).await;
        assert_eq!(updated_list.len(), 3);
        for dto in &dtos {
            assert!(updated_list.contains(dto));
        }
    }
}
