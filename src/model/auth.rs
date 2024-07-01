use thiserror::Error;

#[derive(Debug)]
pub struct AuthRequest {
    user_id: UserId,
}

#[derive(Debug)]
pub struct UserId(uuid::Uuid);

#[derive(Debug, Error)]
pub enum AuthorizationError {
    #[error("The user belonging to this token no longer exists")]
    InvalidCredentials,
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
