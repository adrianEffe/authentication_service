use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheOperationError {
    #[error("Failed to cache token")]
    Save,
    #[error("Failed with error: {reason}")]
    Invalid { reason: String },
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
