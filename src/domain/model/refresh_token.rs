use anyhow::anyhow;
use thiserror::Error;

use super::cache_errors::CacheOperationError;

#[derive(Debug)]
pub struct RefreshRequest {
    token: Token,
}

impl RefreshRequest {
    pub fn new(token: String) -> RefreshRequest {
        RefreshRequest {
            token: Token(token),
        }
    }

    pub fn get_token(&self) -> &str {
        self.token.0.as_str()
    }
}

#[derive(Debug)]
struct Token(String);

#[derive(Debug)]
pub struct RefreshResponse;

#[derive(Debug, Error)]
pub enum RefreshTokenError {
    #[error("Authorization error: {reason}")]
    InvalidCredentials { reason: String },
    #[error("Refresh token not found")]
    MissingCredentials,
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

impl From<CacheOperationError> for RefreshTokenError {
    fn from(value: CacheOperationError) -> Self {
        match value {
            CacheOperationError::Invalid { reason } => {
                RefreshTokenError::InvalidCredentials { reason }
            }
            _ => RefreshTokenError::Unknown(anyhow!("Internal server error")),
        }
    }
}
