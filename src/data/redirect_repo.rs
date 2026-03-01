use async_trait::async_trait;
use sqlx::{Pool, Sqlite};

use crate::{
    data::RedirectRepo,
    model::{RedirectCreationDTO, RedirectDTO, UpdateUrlDTO},
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

    async fn create_redirect(&self, redirect: &RedirectCreationDTO) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO redirects (alias, url, owner) VALUES ($1, $2, $3);")
            .bind(&redirect.redirect.alias)
            .bind(&redirect.redirect.url)
            .bind(&redirect.owner)
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

    async fn read_all_redirects_by_user_id(
        &self,
        user_id: &String,
    ) -> Result<Vec<RedirectDTO>, sqlx::Error> {
        let redirects =
            sqlx::query_as::<_, RedirectDTO>("SELECT alias, url FROM redirects WHERE owner = $1;")
                .bind(&user_id)
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
        model::{RedirectCreationDTO, RedirectDTO, UpdateUrlDTO, User},
    };

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool
    }
    fn get_test_user_data() -> User {
        User {
            id: "some_id_string".to_owned(),
            is_admin: false,
            name: "testuser".to_owned(),
            pwhash: "not_a_pw_hash".to_owned(),
        }
    }
    async fn seed_test_db(pool: &SqlitePool) -> (Vec<RedirectCreationDTO>, User) {
        let owner = get_test_user_data();

        insert_user_into_test_db(&owner, &pool).await;

        let dtos = vec![
            RedirectCreationDTO {
                redirect: RedirectDTO {
                    alias: "somealias".to_string(),
                    url: "https://someurl.com".to_string(),
                },
                owner: owner.id.clone(),
            },
            RedirectCreationDTO {
                redirect: RedirectDTO {
                    alias: "secondalias".to_string(),
                    url: "https://secondurl.com".to_string(),
                },
                owner: owner.id.clone(),
            },
            RedirectCreationDTO {
                redirect: RedirectDTO {
                    alias: "thirdalias".to_string(),
                    url: "https://thirdurl.com".to_string(),
                },
                owner: owner.id.clone(),
            },
        ];

        for dto in &dtos {
            insert_into_test_db(dto.to_owned(), pool).await;
        }
        (dtos, owner)
    }

    async fn read_from_test_db(alias: &str, pool: &SqlitePool) -> Result<RedirectDTO, sqlx::Error> {
        sqlx::query_as::<_, RedirectDTO>("SELECT alias, url FROM redirects WHERE alias = $1;")
            .bind(alias)
            .fetch_one(pool)
            .await
    }

    async fn insert_into_test_db(dto: &RedirectCreationDTO, pool: &SqlitePool) {
        sqlx::query("INSERT INTO redirects (alias, url, owner) VALUES ($1, $2, $3);")
            .bind(&dto.redirect.alias)
            .bind(&dto.redirect.url)
            .bind(&dto.owner)
            .execute(pool)
            .await
            .unwrap();
    }
    async fn insert_user_into_test_db(user: &User, pool: &SqlitePool) {
        sqlx::query("INSERT INTO users (id, name, pwhash, is_admin) VALUES ($1, $2, $3, $5);")
            .bind(&user.id)
            .bind(&user.name)
            .bind(&user.pwhash)
            .bind(user.is_admin)
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
        let (_, owner) = seed_test_db(&pool).await;
        let dto = RedirectCreationDTO {
            redirect: RedirectDTO {
                alias: "somenewalias".to_string(),
                url: "https://someurl.com".to_string(),
            },
            owner: owner.id,
        };

        let result = repo.create_redirect(&dto).await;
        dbg!(result.as_ref().err());
        assert!(result.is_ok());
        let fetched = read_from_test_db(&dto.redirect.alias, &pool).await.unwrap();
        assert_eq!(fetched.alias, "somenewalias");

        assert_eq!(fetched.url, "https://someurl.com");
    }

    #[tokio::test]
    async fn test_create_redirect_with_unknown_user_fails() {
        let pool = setup_test_db().await;
        let repo = RedirectRepoSqliteImpl::new(pool.clone());
        seed_test_db(&pool).await;
        let dto = RedirectCreationDTO {
            redirect: RedirectDTO {
                alias: "somenewalias".to_string(),
                url: "https://someurl.com".to_string(),
            },
            owner: "some_none_existant_user_id".to_owned(),
        };

        let result = repo.create_redirect(&dto).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_redirect_duplicate_alias_fails() {
        let pool = setup_test_db().await;
        let repo = RedirectRepoSqliteImpl::new(pool.clone());

        let (dtos, _) = seed_test_db(&pool).await;

        let duplicate = RedirectCreationDTO {
            redirect: RedirectDTO {
                alias: dtos[0].redirect.alias.clone(),
                url: dtos[0].redirect.url.clone(),
            },
            owner: dtos[0].owner.clone(),
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
        dbg!(result.as_ref().err());
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
        dbg!(result.as_ref().err());
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

        dbg!(result.as_ref().err());
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

        dbg!(result.as_ref().err());
        assert!(result.is_ok());
        let result_list = result.unwrap();
        assert!(result_list.is_empty())
    }

    #[tokio::test]
    async fn test_read_all_redirects_should_return_list() {
        let pool = setup_test_db().await;
        let repo = RedirectRepoSqliteImpl::new(pool.clone());

        let (dtos, _) = seed_test_db(&pool).await;

        let result = repo.read_all_redirects().await;

        dbg!(result.as_ref().err());
        assert!(result.is_ok());

        let result_list = result.unwrap();

        assert_eq!(result_list.len(), 3);

        for result_item in dtos {
            assert!(result_list.contains(&result_item.redirect));
        }
    }

    #[tokio::test]
    async fn test_read_all_redirects_by_user_should_return_limited_list() {
        let pool = setup_test_db().await;
        let repo = RedirectRepoSqliteImpl::new(pool.clone());

        let (seeded_list, _) = seed_test_db(&pool).await;

        let new_user = User {
            id: "new_id_string".to_owned(),
            name: "new_test_user".to_owned(),
            is_admin: false,
            pwhash: "some_pw_hash_but_its_not_a_hash".to_owned(),
        };

        insert_user_into_test_db(&new_user, &pool).await;

        let new_redirect = RedirectCreationDTO {
            redirect: RedirectDTO {
                alias: "the_newest_alias".to_owned(),
                url: "http://url.de".to_owned(),
            },
            owner: new_user.id.clone(),
        };
        insert_into_test_db(&new_redirect, &pool).await;

        let redirects_result = repo.read_all_redirects_by_user_id(&new_user.id).await;
        dbg!(redirects_result.as_ref().err());
        assert!(redirects_result.is_ok());
        let limited_redirect_list = redirects_result.unwrap();
        assert!(!limited_redirect_list.is_empty());
        assert_eq!(limited_redirect_list.len(), 1);
        assert_eq!(limited_redirect_list[0], new_redirect.redirect);
        let full_list = repo.read_all_redirects().await.unwrap();
        assert_eq!(full_list.len(), seeded_list.len() + 1);
    }

    #[tokio::test]
    async fn test_delete_redirect_success() {
        let pool = setup_test_db().await;
        let repo = RedirectRepoSqliteImpl::new(pool.clone());

        let (dtos, _) = seed_test_db(&pool).await;

        let result = repo.delete_redirect_by_alias(&dtos[0].redirect.alias).await;
        dbg!(result.as_ref().err());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
        let updated_list = read_all_from_test_db(&pool).await;
        assert_eq!(updated_list.len(), 2);
        assert!(updated_list.contains(&dtos[1].redirect));
        assert!(updated_list.contains(&dtos[2].redirect));
        assert!(!updated_list.contains(&dtos[0].redirect));
    }

    #[tokio::test]
    async fn test_delete_unknown_redirect_leads_to_no_deletion() {
        let pool = setup_test_db().await;
        let repo = RedirectRepoSqliteImpl::new(pool.clone());

        let (dtos, _) = seed_test_db(&pool).await;

        let result = repo.delete_redirect_by_alias("invalidalias").await;

        dbg!(result.as_ref().err());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        let updated_list = read_all_from_test_db(&pool).await;
        assert_eq!(updated_list.len(), 3);
        for dto in &dtos {
            assert!(updated_list.contains(&dto.redirect));
        }
    }
}
