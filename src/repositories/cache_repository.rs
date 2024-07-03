use anyhow::anyhow;
use redis::{AsyncCommands, Client};

use crate::domain::{
    model::{cache_errors::CacheOperationError, token::TokenDetails, token_uuid::TokenUuid},
    repositories::cache_repository::CacheRepository,
};

#[derive(Debug)]
pub struct RedisCache {
    client: Client,
}

impl RedisCache {
    pub fn new(url: &str) -> RedisCache {
        let client = match Client::open(url) {
            Ok(client) => {
                println!("Connection to redis successful");
                client
            }
            Err(err) => {
                println!("Failed to connect to redis with error: {}", err);
                std::process::exit(1);
            }
        };

        RedisCache { client }
    }
}

impl CacheRepository for RedisCache {
    async fn save_token_data(
        &self,
        token_details: &TokenDetails,
        max_age: i64,
    ) -> Result<(), CacheOperationError> {
        let mut redis_client = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| anyhow!(e).context("Failed to get redis connection"))?;

        redis_client
            .set_ex(
                token_details.token_uuid.to_string(),
                token_details.user_id.to_string(),
                (max_age * 60) as u64,
            )
            .await
            .map_err(|_| CacheOperationError::Save)?;

        Ok(())
    }

    async fn verify_active_session(
        &self,
        token_details: &TokenDetails,
    ) -> Result<(), CacheOperationError> {
        let access_token_uuid = uuid::Uuid::parse_str(&token_details.token_uuid.to_string())
            .map_err(|e| CacheOperationError::Invalid {
                reason: format!("Failed to parse token with error: {}", e),
            })?;

        let mut redis_client = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| anyhow!(e).context("Failed to get redis connection"))?;

        redis_client
            .get::<_, String>(access_token_uuid.to_string())
            .await
            .map_err(|_| CacheOperationError::Invalid {
                reason: "Token is invalid or session has expired".to_string(),
            })?;
        Ok(())
    }

    async fn delete_token(&self, token_uuid: &TokenUuid) -> Result<(), CacheOperationError> {
        let mut redis_client = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| anyhow!(e).context("Failed to get redis connection"))?;

        redis_client
            .del(token_uuid.get_string())
            .await
            .map_err(|e| anyhow!(e).context("Failed to delete token from redis"))?;

        Ok(())
    }
}
