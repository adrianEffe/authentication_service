use serde::{Deserialize, Serialize};

//TODO: add getters and setters
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub token_uuid: String,
    pub exp: i64,
    pub iat: i64,
    pub nbf: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenDetails {
    pub token: Option<String>,
    pub token_uuid: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub expires_in: Option<i64>,
}

#[derive(Debug)]
pub struct CacheToken {
    pub token_uuid: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub max_age: i64,
}

impl CacheToken {
    pub fn new(token_uuid: uuid::Uuid, user_id: uuid::Uuid, max_age: i64) -> CacheToken {
        CacheToken {
            token_uuid,
            user_id,
            max_age,
        }
    }
}
