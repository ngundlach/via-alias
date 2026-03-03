use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{
    data::UserRegistrationTokenRepo,
    model::UserRegistrationToken,
    service::{DbServiceError, validate_registration_token},
};

pub(crate) struct UserRegistrationTokenInMemoryImpl {
    token_store: Arc<RwLock<TokenStore>>,
    cancel_token: CancellationToken,
}
struct TokenStore {
    tokens: HashMap<String, UserRegistrationToken>,
    expirations: BTreeMap<u64, HashSet<String>>,
}

impl Drop for UserRegistrationTokenInMemoryImpl {
    fn drop(&mut self) {
        self.cancel_token.cancel();
    }
}

impl UserRegistrationTokenInMemoryImpl {
    pub fn new() -> Self {
        let cancel_token = CancellationToken::new();
        Self {
            cancel_token,
            token_store: Arc::new(RwLock::new(TokenStore {
                tokens: HashMap::new(),
                expirations: BTreeMap::new(),
            })),
        }
    }

    pub(crate) fn start_cleanup(self: &Arc<Self>, cleanup_interval: Duration) {
        let repo = self.clone();
        let cancel_token = self.cancel_token.clone();
        tokio::spawn(async move {
            let mut timer = tokio::time::interval(cleanup_interval);

            loop {
                tokio::select! {
                    _ = timer.tick() => {
                         repo.delete_expired_tokens().await;
                     }
                     _ = cancel_token.cancelled() => {
                         break;
                     }
                }
            }
        });
    }

    async fn delete_expired_tokens(&self) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is before Unix epoch")
            .as_secs();

        let mut guard = self.token_store.write().await;

        let expired_keys: Vec<u64> = guard
            .expirations
            .range(..=current_time)
            .map(|(k, _)| *k)
            .collect();

        for key in expired_keys {
            if let Some(bucket) = guard.expirations.remove(&key) {
                for token in bucket {
                    guard.tokens.remove(&token);
                }
            }
        }
    }
}

#[async_trait]
impl UserRegistrationTokenRepo for UserRegistrationTokenInMemoryImpl {
    async fn create_user_registration_token(
        &self,
    ) -> Result<UserRegistrationToken, DbServiceError> {
        let exp_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is before Unix epoch")
            .checked_add(Duration::from_mins(30))
            .expect("timestamp overflow")
            .as_secs();

        let new_user_token = UserRegistrationToken {
            registration_token: Uuid::new_v4().to_string(),
            exp_at,
        };
        let mut guard = self.token_store.write().await;
        guard.tokens.insert(
            new_user_token.registration_token.clone(),
            new_user_token.clone(),
        );

        let expires_at = new_user_token.exp_at;
        guard
            .expirations
            .entry(expires_at)
            .or_default()
            .insert(new_user_token.registration_token.clone());
        Ok(new_user_token)
    }

    async fn read_token(&self, token_str: &str) -> Result<UserRegistrationToken, DbServiceError> {
        let token = {
            let guard = self.token_store.read().await;

            let token = guard
                .tokens
                .get(token_str)
                .ok_or(DbServiceError::NotFoundError)?;
            token.clone()
        };
        if validate_registration_token(&token).is_err() {
            let _ = self
                .delete_user_registration_token(&token.registration_token)
                .await;
            return Err(DbServiceError::TokenInvalid);
        }

        Ok(token)
    }

    async fn delete_user_registration_token(&self, token: &str) -> Result<(), DbServiceError> {
        let mut guard = self.token_store.write().await;
        let (_, token_data) = guard
            .tokens
            .remove_entry(token)
            .ok_or(DbServiceError::NotFoundError)?;

        if let Some(token_bucket) = guard.expirations.get_mut(&token_data.exp_at) {
            token_bucket.remove(&token_data.registration_token);
            if token_bucket.is_empty() {
                guard.expirations.remove(&token_data.exp_at);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use std::{
        sync::Arc,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    use tokio::task::JoinSet;

    use crate::{
        data::{UserRegistrationTokenInMemoryImpl, UserRegistrationTokenRepo},
        model::UserRegistrationToken,
        service::DbServiceError,
    };
    fn future_time() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .checked_add(Duration::from_hours(1))
            .unwrap()
            .as_secs()
    }

    #[tokio::test]
    async fn test_inserting_new_registration_token_success() {
        let token_repo = UserRegistrationTokenInMemoryImpl::new();
        let token = token_repo.create_user_registration_token().await;
        dbg!(token.as_ref().err());
        assert!(token.is_ok());
        let token = token.unwrap();
        let guard = token_repo.token_store.read().await;
        assert_eq!(guard.tokens.len(), 1);
        assert!(guard.tokens.contains_key(&token.registration_token));
        assert_eq!(
            guard.tokens.get(&token.registration_token).unwrap().clone(),
            token
        );
        let tokens_in_expired = guard.expirations.get(&token.exp_at).unwrap();
        assert_eq!(guard.expirations.len(), 1);
        assert_eq!(tokens_in_expired.len(), 1);
        assert_eq!(
            tokens_in_expired
                .get(&token.registration_token)
                .unwrap()
                .clone(),
            token.registration_token
        );
    }

    #[tokio::test]
    async fn test_read_token_returns_correct_token() {
        let store = UserRegistrationTokenInMemoryImpl::new();
        let created = store.create_user_registration_token().await.unwrap();

        let read = store.read_token(&created.registration_token).await;
        assert!(read.is_ok());
        assert_eq!(read.unwrap(), created);
    }

    #[tokio::test]
    async fn test_read_token_not_found_returns_error() {
        let store = UserRegistrationTokenInMemoryImpl::new();
        let result = store.read_token("nonexistent-token").await;
        assert!(matches!(result, Err(DbServiceError::NotFoundError)));
    }

    #[tokio::test]
    async fn test_read_expired_token_returns_error_and_deletes_token() {
        let store = UserRegistrationTokenInMemoryImpl::new();

        let expired_token = UserRegistrationToken {
            registration_token: "valid_token".to_string(),
            exp_at: 1,
        };
        {
            let mut guard = store.token_store.write().await;
            guard.tokens.insert(
                expired_token.registration_token.clone(),
                expired_token.clone(),
            );
            let bucket = guard.expirations.entry(expired_token.exp_at).or_default();
            bucket.insert(expired_token.registration_token);
        }
        let result = store.read_token("valid_token").await;
        assert!(result.is_err());
        assert!(matches!(result, Err(DbServiceError::TokenInvalid)));
        let guard = store.token_store.read().await;
        assert_eq!(guard.tokens.len(), 0);
        assert_eq!(guard.expirations.len(), 0);
    }

    #[tokio::test]
    async fn test_delete_token_removes_from_both_maps() {
        let store = UserRegistrationTokenInMemoryImpl::new();
        let token = store.create_user_registration_token().await.unwrap();

        let result = store
            .delete_user_registration_token(&token.registration_token)
            .await;
        assert!(result.is_ok());

        let guard = store.token_store.read().await;
        assert!(!guard.tokens.contains_key(&token.registration_token));

        assert!(guard.expirations.is_empty());
    }

    #[tokio::test]
    async fn test_delete_token_is_no_longer_readable() {
        let store = UserRegistrationTokenInMemoryImpl::new();
        let token = store.create_user_registration_token().await.unwrap();
        store
            .delete_user_registration_token(&token.registration_token)
            .await
            .unwrap();

        let result = store.read_token(&token.registration_token).await;
        assert!(matches!(result, Err(DbServiceError::NotFoundError)));
    }

    #[tokio::test]
    async fn test_delete_nonexistent_token_returns_error() {
        let store = UserRegistrationTokenInMemoryImpl::new();
        let fake_token = UserRegistrationToken {
            registration_token: "does-not-exist".to_string(),
            exp_at: future_time(),
        };

        let result = store
            .delete_user_registration_token(&fake_token.registration_token)
            .await;
        assert!(matches!(result, Err(DbServiceError::NotFoundError)));
    }

    #[tokio::test]
    async fn test_delete_one_token_leaves_other_intact() {
        let store = UserRegistrationTokenInMemoryImpl::new();
        let token_a = store.create_user_registration_token().await.unwrap();
        let token_b = store.create_user_registration_token().await.unwrap();

        store
            .delete_user_registration_token(&token_a.registration_token)
            .await
            .unwrap();

        assert!(store.read_token(&token_a.registration_token).await.is_err());

        let result = store.read_token(&token_b.registration_token).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), token_b);
    }

    #[tokio::test]
    async fn test_multiple_tokens_same_expiry_bucket_cleaned_up_correctly() {
        let store = UserRegistrationTokenInMemoryImpl::new();

        let shared_exp: u64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .checked_add(Duration::from_hours(1))
            .unwrap()
            .as_secs();

        let token_a = UserRegistrationToken {
            registration_token: "token-a".to_string(),
            exp_at: shared_exp,
        };
        let token_b = UserRegistrationToken {
            registration_token: "token-b".to_string(),
            exp_at: shared_exp,
        };
        {
            let mut guard = store.token_store.write().await;
            guard
                .tokens
                .insert(token_a.registration_token.clone(), token_a.clone());
            guard
                .tokens
                .insert(token_b.registration_token.clone(), token_b.clone());
            let bucket = guard.expirations.entry(shared_exp).or_default();
            bucket.insert(token_a.registration_token.clone());
            bucket.insert(token_b.registration_token.clone());
        }

        store
            .delete_user_registration_token(&token_a.registration_token)
            .await
            .unwrap();

        let guard = store.token_store.read().await;
        let bucket = guard.expirations.get(&shared_exp);
        assert!(bucket.is_some(), "bucket should still exist");
        let bucket = bucket.unwrap();
        assert_eq!(bucket.len(), 1);
        assert!(bucket.contains(&token_b.registration_token));
    }

    #[tokio::test]
    async fn test_concurrent_token_creation_all_unique() {
        let store = Arc::new(UserRegistrationTokenInMemoryImpl::new());
        let n = 50;
        let mut join_set = JoinSet::new();
        for _ in 0..n {
            let s = Arc::clone(&store);
            join_set.spawn(async move { s.create_user_registration_token().await.unwrap() });
        }

        let mut tokens = vec![];
        while let Some(result) = join_set.join_next().await {
            tokens.push(result.unwrap());
        }

        let unique: std::collections::HashSet<_> =
            tokens.iter().map(|t| &t.registration_token).collect();
        assert_eq!(unique.len(), n);

        let guard = store.token_store.read().await;
        assert_eq!(guard.tokens.len(), n);
    }

    #[tokio::test]
    async fn test_delete_expired_tokens_empty_store_does_not_panic() {
        let store = UserRegistrationTokenInMemoryImpl::new();
        store.delete_expired_tokens().await;
        let guard = store.token_store.read().await;
        assert!(guard.tokens.is_empty());
        assert!(guard.expirations.is_empty());
    }

    #[tokio::test]
    async fn test_delete_expired_tokens_leaves_valid_tokens_intact() {
        let store = UserRegistrationTokenInMemoryImpl::new();

        let future_exp = future_time();
        let token = UserRegistrationToken {
            registration_token: "valid-token".to_string(),
            exp_at: future_exp,
        };
        {
            let mut guard = store.token_store.write().await;
            guard
                .tokens
                .insert(token.registration_token.clone(), token.clone());
            guard
                .expirations
                .entry(future_exp)
                .or_default()
                .insert(token.registration_token.clone());
        }

        store.delete_expired_tokens().await;

        let guard = store.token_store.read().await;
        assert_eq!(guard.tokens.len(), 1);
        assert!(guard.tokens.contains_key(&token.registration_token));
        assert_eq!(guard.expirations.len(), 1);
    }

    #[tokio::test]
    async fn test_delete_expired_tokens_removes_expired_token() {
        let store = UserRegistrationTokenInMemoryImpl::new();

        let past_exp: u64 = 1;
        let token = UserRegistrationToken {
            registration_token: "expired-token".to_string(),
            exp_at: past_exp,
        };
        {
            let mut guard = store.token_store.write().await;
            guard
                .tokens
                .insert(token.registration_token.clone(), token.clone());
            guard
                .expirations
                .entry(past_exp)
                .or_default()
                .insert(token.registration_token.clone());
        }

        store.delete_expired_tokens().await;

        let guard = store.token_store.read().await;
        assert!(guard.tokens.is_empty());
        assert!(guard.expirations.is_empty());
    }

    #[tokio::test]
    async fn test_delete_expired_tokens_only_removes_expired_in_mixed_store() {
        let store = UserRegistrationTokenInMemoryImpl::new();

        let past_exp: u64 = 1;
        let future_exp = future_time();

        let expired_token = UserRegistrationToken {
            registration_token: "expired-token".to_string(),
            exp_at: past_exp,
        };
        let valid_token = UserRegistrationToken {
            registration_token: "valid-token".to_string(),
            exp_at: future_exp,
        };
        {
            let mut guard = store.token_store.write().await;
            guard.tokens.insert(
                expired_token.registration_token.clone(),
                expired_token.clone(),
            );
            guard
                .tokens
                .insert(valid_token.registration_token.clone(), valid_token.clone());
            guard
                .expirations
                .entry(past_exp)
                .or_default()
                .insert(expired_token.registration_token.clone());
            guard
                .expirations
                .entry(future_exp)
                .or_default()
                .insert(valid_token.registration_token.clone());
        }

        store.delete_expired_tokens().await;

        let guard = store.token_store.read().await;
        assert_eq!(guard.tokens.len(), 1);
        assert!(!guard.tokens.contains_key(&expired_token.registration_token));
        assert!(guard.tokens.contains_key(&valid_token.registration_token));
        assert_eq!(guard.expirations.len(), 1);
        assert!(guard.expirations.contains_key(&future_exp));
    }

    #[tokio::test]
    async fn test_delete_expired_tokens_removes_multiple_expired_buckets() {
        let store = UserRegistrationTokenInMemoryImpl::new();

        let past_exps: Vec<u64> = vec![1, 2, 3];
        let future_exp: u64 = future_time();

        let expired_tokens: Vec<UserRegistrationToken> = past_exps
            .iter()
            .enumerate()
            .map(|(i, &exp)| UserRegistrationToken {
                registration_token: format!("expired-token-{}", i),
                exp_at: exp,
            })
            .collect();

        let valid_token = UserRegistrationToken {
            registration_token: "valid-token".to_string(),
            exp_at: future_exp,
        };

        {
            let mut guard = store.token_store.write().await;
            for token in &expired_tokens {
                guard
                    .tokens
                    .insert(token.registration_token.clone(), token.clone());
                guard
                    .expirations
                    .entry(token.exp_at)
                    .or_default()
                    .insert(token.registration_token.clone());
            }
            guard
                .tokens
                .insert(valid_token.registration_token.clone(), valid_token.clone());
            guard
                .expirations
                .entry(future_exp)
                .or_default()
                .insert(valid_token.registration_token.clone());
        }

        store.delete_expired_tokens().await;

        let guard = store.token_store.read().await;
        assert_eq!(guard.tokens.len(), 1);
        assert!(guard.tokens.contains_key(&valid_token.registration_token));
        assert_eq!(guard.expirations.len(), 1);
        assert!(guard.expirations.contains_key(&future_exp));
    }
}
