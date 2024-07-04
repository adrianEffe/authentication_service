use std::future::Future;

use crate::domain::model::{
    cache_errors::CacheOperationError,
    token::{CacheToken, TokenDetails},
    token_uuid::TokenUuid,
};

pub trait CacheRepository: Send + Sync + 'static {
    fn save_token_data(
        &self,
        token: &CacheToken,
    ) -> impl Future<Output = Result<(), CacheOperationError>> + Send;

    fn save_tokens_data(
        &self,
        access_token: &CacheToken,
        refresh_token: &CacheToken,
    ) -> impl Future<Output = Result<(), CacheOperationError>> + Send;

    fn verify_active_session(
        &self,
        token_details: &TokenDetails,
    ) -> impl Future<Output = Result<(), CacheOperationError>> + Send;

    fn delete_token(
        &self,
        token_uuid: &TokenUuid,
    ) -> impl Future<Output = Result<(), CacheOperationError>> + Send;
}
