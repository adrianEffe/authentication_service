use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheOperationError {
    #[error("Failed to cache token")]
    Save,
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
