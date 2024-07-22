use std::future::Future;

use crate::domain::model::{
    cache_errors::CacheOperationError,
    token::{CacheToken, TokenDetails},
    token_uuid::TokenUuid,
};

/// Trait defining the contract for cache-related operations.
///
/// The `CacheRepository` trait specifies the necessary methods for interacting with a cache
/// storage system. Implementing this trait allows for operations such as saving token data,
/// verifying active sessions, and deleting tokens.
///
/// # Requirements
///
/// Any struct that implements the `CacheRepository` trait must be `Send`, `Sync`,
/// and have a `'static` lifetime. This ensures that instances of the implementing
/// struct can be safely shared across threads and have a static lifetime.
///
/// # Errors
///
/// The methods in this trait return a `Result` with `()` on success or a `CacheOperationError` on failure.
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
