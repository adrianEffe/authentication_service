use anyhow::anyhow;
use thiserror::Error;

use super::cache_errors::CacheOperationError;

#[derive(Debug)]
pub struct AuthRequest {
    pub access_token: AccessToken,
}

impl AuthRequest {
    pub fn new(access_token: String) -> AuthRequest {
        AuthRequest {
            access_token: AccessToken(access_token),
        }
    }
}

#[derive(Debug)]
pub struct AccessToken(String);

impl AccessToken {
    pub fn get(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Error)]
pub enum AuthorizationError {
    #[error("Authorization error: {reason}")]
    InvalidCredentials { reason: String },
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

impl From<CacheOperationError> for AuthorizationError {
    fn from(value: CacheOperationError) -> Self {
        match value {
            CacheOperationError::Invalid { reason } => {
                AuthorizationError::InvalidCredentials { reason }
            }
            _ => AuthorizationError::Unknown(anyhow!("Internal server error")),
        }
    }
}
