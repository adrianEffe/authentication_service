use thiserror::Error;

#[derive(Debug)]
pub struct AuthRequest {
    pub user_id: UserId,
}

#[derive(Debug)]
pub struct UserId(uuid::Uuid);

impl UserId {
    pub fn get(&self) -> &uuid::Uuid {
        &self.0
    }
}

#[derive(Debug, Error)]
pub enum AuthorizationError {
    #[error("The user belonging to this token no longer exists")]
    InvalidCredentials,
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
