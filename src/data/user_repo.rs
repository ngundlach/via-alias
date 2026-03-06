use async_trait::async_trait;
use sqlx::{Pool, Sqlite};

use crate::{
    data::{DeletedResources, UserRepo, UserRepoError},
    model::User,
};

pub struct UserRepoSqliteImpl {
    db: Pool<Sqlite>,
}

impl UserRepoSqliteImpl {
    pub fn new(db: Pool<Sqlite>) -> Self {
        UserRepoSqliteImpl { db }
    }
}

#[async_trait]
impl UserRepo for UserRepoSqliteImpl {
    async fn read_user_by_name(&self, name: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT id, name, pwhash, is_admin FROM users WHERE name = $1;")
            .bind(name)
            .fetch_one(&self.db)
            .await
    }

    async fn read_user_by_id(&self, id: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT id, name, pwhash, is_admin FROM users WHERE id = $1;")
            .bind(id)
            .fetch_one(&self.db)
            .await
    }

    async fn read_users(&self) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT id, name, pwhash, is_admin FROM users;")
            .fetch_all(&self.db)
            .await
    }

    async fn create_user(&self, user: &User) -> Result<User, sqlx::Error> {
        sqlx::query("INSERT INTO users (id, name, pwhash, is_admin) VALUES ($1, $2, $3, $4);")
            .bind(&user.id)
            .bind(&user.name)
            .bind(&user.pwhash)
            .bind(user.is_admin)
            .execute(&self.db)
            .await?;
        Ok(user.clone())
    }

    async fn count_user_with_is_admin(&self) -> Result<i64, sqlx::Error> {
        let result: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE is_admin = 1")
            .fetch_one(&self.db)
            .await?;
        Ok(result)
    }

    async fn update_user_by_id(&self, user: &User) -> Result<u64, sqlx::Error> {
        let res = sqlx::query("UPDATE users SET name=$2, is_admin=$3, pwhash = $4 WHERE id=$1;")
            .bind(&user.id)
            .bind(&user.name)
            .bind(user.is_admin)
            .bind(&user.pwhash)
            .execute(&self.db)
            .await?
            .rows_affected();
        Ok(res)
    }

    async fn delete_user_by_id(&self, user_id: &str) -> Result<DeletedResources, UserRepoError> {
        let mut tx = self.db.begin().await?;
        let is_admin: bool = sqlx::query_scalar("SELECT is_admin FROM users where id = $1;")
            .bind(&user_id)
            .fetch_one(&mut *tx)
            .await?;

        if is_admin {
            return Err(UserRepoError::IsAdmin);
        }

        let deleted_redirects = sqlx::query("DELETE FROM redirects WHERE owner = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?
            .rows_affected();

        let deleted_users = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?
            .rows_affected();

        tx.commit().await?;
        Ok(DeletedResources {
            affected_user_rows: deleted_users,
            affected_resources: deleted_redirects,
        })
    }
}

#[cfg(test)]
mod tests {
    use sqlx::SqlitePool;
    use uuid::Uuid;

    use crate::{
        data::{UserRepo, UserRepoError, UserRepoSqliteImpl},
        model::{Redirect, User},
    };

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool
    }

    async fn seed_test_db(pool: &SqlitePool) -> Vec<User> {
        let users = vec![
            User {
                id: "1".to_owned(),
                name: "luke earthwalker".to_owned(),
                pwhash: "some_pw_hash".to_string(),
                is_admin: false,
            },
            User {
                id: "2".to_owned(),
                name: "Darth Nin".to_owned(),
                pwhash: "some_pw_other_hash".to_string(),
                is_admin: true,
            },
            User {
                id: "3".to_owned(),
                name: "Lando".to_owned(),
                pwhash: "some_third_pw_hash".to_string(),
                is_admin: false,
            },
        ];

        for u in &users {
            insert_into_test_db(u.to_owned(), pool).await;
        }
        users
    }

    async fn read_from_test_db(user_id: &str, pool: &SqlitePool) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1;")
            .bind(user_id)
            .fetch_one(pool)
            .await
    }

    async fn insert_into_test_db(user: User, pool: &SqlitePool) {
        sqlx::query("INSERT INTO users (id, name, pwhash, is_admin) VALUES ($1, $2, $3, $4);")
            .bind(&user.id)
            .bind(&user.name)
            .bind(&user.pwhash)
            .bind(user.is_admin)
            .execute(pool)
            .await
            .unwrap();
    }

    async fn insert_redirect_into_test_db(redirect: &Redirect, pool: &SqlitePool) {
        sqlx::query("INSERT INTO redirects (id, alias, url, owner) VALUES ($1, $2, $3, $4);")
            .bind(&redirect.id)
            .bind(&redirect.alias)
            .bind(&redirect.url)
            .bind(&redirect.owner)
            .execute(pool)
            .await
            .unwrap();
    }

    async fn read_redirects_from_test_db(pool: &SqlitePool) -> Vec<Redirect> {
        sqlx::query_as::<_, Redirect>("SELECT * FROM redirects;")
            .fetch_all(pool)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_create_user_success() {
        let pool = setup_test_db().await;
        let repo = UserRepoSqliteImpl::new(pool.clone());

        let user = User {
            id: "10".to_owned(),
            name: "some_user".to_string(),
            pwhash: "a_pw_hash".to_string(),
            is_admin: false,
        };

        let result = repo.create_user(&user).await;
        dbg!(result.as_ref().err());
        assert!(result.is_ok());
        let fetched = read_from_test_db(&user.id, &pool).await.unwrap();
        assert_eq!(user, fetched);
    }

    #[tokio::test]
    async fn test_create_user_duplicate_fails() {
        let pool = setup_test_db().await;
        let repo = UserRepoSqliteImpl::new(pool.clone());

        let users = seed_test_db(&pool).await;

        let duplicate = users[0].clone();

        let result = repo.create_user(&duplicate).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(sqlx::Error::Database(_))));
    }

    #[tokio::test]
    async fn test_update_user_success() {
        let pool = setup_test_db().await;
        let repo = UserRepoSqliteImpl::new(pool.clone());

        let users = seed_test_db(&pool).await;
        let mut updated_user = users[0].clone();
        updated_user.name = "Steven".to_owned();
        let result = repo.update_user_by_id(&updated_user).await;
        dbg!(result.as_ref().err());
        assert!(result.is_ok());

        let updated = read_from_test_db(&updated_user.id, &pool).await.unwrap();

        assert_eq!(updated_user, updated);
    }

    #[tokio::test]
    async fn test_update_user_pw_success() {
        let pool = setup_test_db().await;
        let repo = UserRepoSqliteImpl::new(pool.clone());

        let users = seed_test_db(&pool).await;
        let mut updated_user = users[0].clone();
        updated_user.pwhash = "not_a_pw_hash_but_changed".to_owned();
        let result = repo.update_user_by_id(&updated_user).await;
        dbg!(result.as_ref().err());
        assert!(result.is_ok());

        let updated = read_from_test_db(&updated_user.id, &pool).await.unwrap();

        assert_eq!(updated_user, updated);
    }

    #[tokio::test]
    async fn test_count_user_with_admin_role() {
        let pool = setup_test_db().await;
        let repo = UserRepoSqliteImpl::new(pool.clone());

        seed_test_db(&pool).await;

        let count = repo.count_user_with_is_admin().await;
        dbg!(count.as_ref().err());
        assert!(count.is_ok());
        assert_eq!(count.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_read_users_should_return_list() {
        let pool = setup_test_db().await;
        let repo = UserRepoSqliteImpl::new(pool.clone());

        let users = seed_test_db(&pool).await;

        let result = repo.read_users().await;
        dbg!(result.as_ref().err());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), users);
    }

    #[tokio::test]
    async fn test_read_user_by_name_succeeds() {
        let pool = setup_test_db().await;
        let repo = UserRepoSqliteImpl::new(pool.clone());

        let users = seed_test_db(&pool).await;

        let result = repo.read_user_by_name(&users[0].name).await;
        dbg!(result.as_ref().err());
        assert!(result.is_ok());
        let expected = users[0].clone();
        assert_eq!(result.unwrap(), expected);
    }

    #[tokio::test]
    async fn test_read_user_by_name_fails() {
        let pool = setup_test_db().await;
        let repo = UserRepoSqliteImpl::new(pool.clone());

        seed_test_db(&pool).await;

        let result = repo.read_user_by_name("unknown").await;
        assert!(result.is_err());
        assert!(matches!(result, Err(sqlx::Error::RowNotFound)));
    }

    #[tokio::test]
    async fn test_read_user_by_id_succeeds() {
        let pool = setup_test_db().await;
        let repo = UserRepoSqliteImpl::new(pool.clone());

        let users = seed_test_db(&pool).await;

        let result = repo.read_user_by_id(&users[0].id).await;
        dbg!(result.as_ref().err());
        assert!(result.is_ok());
        let expected = users[0].clone();
        assert_eq!(result.unwrap(), expected);
    }

    #[tokio::test]
    async fn test_read_user_by_id_fails() {
        let pool = setup_test_db().await;
        let repo = UserRepoSqliteImpl::new(pool.clone());

        seed_test_db(&pool).await;

        let result = repo.read_user_by_id(&Uuid::new_v4().to_string()).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(sqlx::Error::RowNotFound)));
    }

    #[tokio::test]
    async fn test_delete_user_should_delete_user_and_redirects_success() {
        let pool = setup_test_db().await;
        let repo = UserRepoSqliteImpl::new(pool.clone());

        let users = seed_test_db(&pool).await;
        let redirects = vec![
            Redirect {
                id: Uuid::new_v4().to_string(),
                alias: "somealias".to_owned(),
                url: "someurl".to_owned(),
                owner: users[0].clone().id,
            },
            Redirect {
                id: Uuid::new_v4().to_string(),
                alias: "somesecondalias".to_owned(),
                url: "somesecondurl".to_owned(),
                owner: users[0].clone().id,
            },
            Redirect {
                id: Uuid::new_v4().to_string(),
                alias: "someotheralias".to_owned(),
                url: "someotherurl".to_owned(),
                owner: users[1].clone().id,
            },
        ];
        for red in &redirects {
            insert_redirect_into_test_db(red, &pool).await;
        }
        let redirects_from_db = read_redirects_from_test_db(&pool).await;
        assert_eq!(redirects_from_db, redirects);
        let res = repo.delete_user_by_id(&users[0].id).await;
        dbg!(res.as_ref().err());
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res.affected_user_rows, 1);
        assert_eq!(res.affected_resources, 2);

        let updated_redirects_from_db = read_redirects_from_test_db(&pool).await;
        assert_eq!(updated_redirects_from_db.len(), 1);
        assert!(updated_redirects_from_db.contains(&redirects[2]));
    }

    #[tokio::test]
    async fn test_delete_admin_should_return_err_and_delete_nothing() {
        let pool = setup_test_db().await;
        let repo = UserRepoSqliteImpl::new(pool.clone());

        let users = seed_test_db(&pool).await;
        let admin = User {
            id: Uuid::new_v4().to_string(),
            name: "adminiman".to_owned(),
            pwhash: "somehash".to_string(),
            is_admin: true,
        };
        insert_into_test_db(admin.clone(), &pool).await;

        let redirects = vec![
            Redirect {
                id: Uuid::new_v4().to_string(),
                alias: "somealias".to_owned(),
                url: "someurl".to_owned(),
                owner: admin.clone().id,
            },
            Redirect {
                id: Uuid::new_v4().to_string(),
                alias: "somesecondalias".to_owned(),
                url: "somesecondurl".to_owned(),
                owner: users[0].clone().id,
            },
            Redirect {
                id: Uuid::new_v4().to_string(),
                alias: "someotheralias".to_owned(),
                url: "someotherurl".to_owned(),
                owner: users[1].clone().id,
            },
        ];
        for red in &redirects {
            insert_redirect_into_test_db(red, &pool).await;
        }
        let redirects_from_db = read_redirects_from_test_db(&pool).await;
        assert_eq!(redirects_from_db, redirects);
        let res = repo.delete_user_by_id(&admin.id).await;
        assert!(res.is_err());
        assert!(matches!(res.err().unwrap(), UserRepoError::IsAdmin));

        let updated_redirects_from_db = read_redirects_from_test_db(&pool).await;
        assert_eq!(updated_redirects_from_db.len(), 3);
        assert_eq!(redirects, updated_redirects_from_db);
    }
}
