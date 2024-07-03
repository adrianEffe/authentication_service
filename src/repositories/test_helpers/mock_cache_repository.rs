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
}
