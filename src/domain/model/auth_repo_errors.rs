use thiserror::Error;

use super::user_email::UserEmail;

#[derive(Debug, Error)]
pub enum AuthRepositoryError {
    #[error("User with email {email} already exists")]
    Duplicate { email: UserEmail },
    #[error("Database error: {reason}")]
    Database { reason: String },
    #[error("Authorization error: {reason}")]
    InvalidCredentials { reason: String },
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
