#[cfg(test)]
pub mod test_helpers {
    use std::{mem, ops::DerefMut, sync::Arc};

    use anyhow::anyhow;
    use tokio::sync::Mutex;

    use crate::domain::{
        model::{cache_errors::CacheOperationError, token::TokenDetails, token_uuid::TokenUuid},
        repositories::cache_repository::CacheRepository,
    };

    pub struct MockCacheRepository {
        pub save_token_data_result: Arc<Mutex<Result<(), CacheOperationError>>>,
        pub verify_active_session_result: Arc<Mutex<Result<(), CacheOperationError>>>,
        pub delete_token_result: Arc<Mutex<Result<(), CacheOperationError>>>,
    }

    impl CacheRepository for MockCacheRepository {
        async fn save_token_data(
            &self,
            _token_details: &TokenDetails,
            _max_age: i64,
        ) -> Result<(), CacheOperationError> {
            let mut guard = self.save_token_data_result.lock().await;
            let mut result = Err(CacheOperationError::Unknown(anyhow!("substitute error")));
            mem::swap(guard.deref_mut(), &mut result);
            result
        }

        async fn verify_active_session(
            &self,
            _token_details: &TokenDetails,
        ) -> Result<(), CacheOperationError> {
            let mut guard = self.verify_active_session_result.lock().await;
            let mut result = Err(CacheOperationError::Unknown(anyhow!("substitute error")));
            mem::swap(guard.deref_mut(), &mut result);
            result
        }

        async fn delete_token(&self, _token_uuid: &TokenUuid) -> Result<(), CacheOperationError> {
            let mut guard = self.delete_token_result.lock().await;
            let mut result = Err(CacheOperationError::Unknown(anyhow!("substitute error")));
            mem::swap(guard.deref_mut(), &mut result);
            result
        }
    }

    impl MockCacheRepository {
        pub fn success() -> MockCacheRepository {
            let save_token_data_result = Arc::new(Mutex::new(Ok(())));
            let verify_active_session_result = Arc::new(Mutex::new(Ok(())));
            let delete_token_result = Arc::new(Mutex::new(Ok(())));

            MockCacheRepository {
                save_token_data_result,
                verify_active_session_result,
                delete_token_result,
            }
        }

        pub fn failure() -> MockCacheRepository {
            let save_token_data_result = Arc::new(Mutex::new(Err(CacheOperationError::Unknown(
                anyhow!("save token data result error"),
            ))));
            let verify_active_session_result = Arc::new(Mutex::new(Err(
                CacheOperationError::Unknown(anyhow!("verify active session result error")),
            )));
            let delete_token_result = Arc::new(Mutex::new(Err(CacheOperationError::Unknown(
                anyhow!("delete token result error"),
            ))));

            MockCacheRepository {
                save_token_data_result,
                verify_active_session_result,
                delete_token_result,
            }
        }
    }

    #[tokio::test]
    async fn test_cache_repository_success_cases() {
        let uuid = uuid::Uuid::new_v4();
        let token = TokenDetails {
            token: None,
            token_uuid: uuid,
            user_id: uuid,
            expires_in: None,
        };

        let mock_repo = MockCacheRepository::success();

        let result = mock_repo.save_token_data(&token, 10).await;
        assert!(result.is_ok());

        let result = mock_repo.verify_active_session(&token).await;
        assert!(result.is_ok());

        let result = mock_repo.delete_token(&TokenUuid::new(uuid)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cache_repository_failure_cases() {
        let uuid = uuid::Uuid::new_v4();
        let token = TokenDetails {
            token: None,
            token_uuid: uuid,
            user_id: uuid,
            expires_in: None,
        };

        let mock_repo = MockCacheRepository::failure();

        let result = mock_repo.save_token_data(&token, 10).await;
        assert!(result.is_err());

        let result = mock_repo.verify_active_session(&token).await;
        assert!(result.is_err());

        let result = mock_repo.delete_token(&TokenUuid::new(uuid)).await;
        assert!(result.is_err());
    }
}
