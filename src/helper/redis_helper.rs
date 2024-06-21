use crate::{application::AppState, model::token::TokenDetails};

use anyhow::Result;
use redis::{AsyncCommands, Client};
use std::sync::Arc;

pub async fn save_token_data(
    data: &Arc<AppState>,
    token_details: &TokenDetails,
    max_age: i64,
) -> Result<()> {
    let mut redis_client = data.redis.get_multiplexed_async_connection().await?;

    redis_client
        .set_ex(
            token_details.token_uuid.to_string(),
            token_details.user_id.to_string(),
            (max_age * 60) as u64,
        )
        .await?;
    Ok(())
}

pub async fn delete_token_data(redis: &Client, access_token_uuid: &str) -> Result<()> {
    let mut redis_client = redis.get_multiplexed_async_connection().await?;
    redis_client.del(access_token_uuid.to_string()).await?;

    Ok(())
}
