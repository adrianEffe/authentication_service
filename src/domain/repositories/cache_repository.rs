use std::future::Future;

use crate::model::cache_errors::CacheOperationError;
use crate::model::token::TokenDetails;

pub trait CacheRepository: Send + Sync + 'static {
    fn save_token_data(
        &self,
        token_details: &TokenDetails,
        max_age: i64,
    ) -> impl Future<Output = Result<(), CacheOperationError>> + Send;
}
