use thiserror::Error;

#[derive(Debug)]
pub struct AuthRequest {
    pub user_id: UserId,
}

impl AuthRequest {
    pub fn new(user_id: uuid::Uuid) -> AuthRequest {
        AuthRequest {
            user_id: UserId::new(user_id),
        }
    }
}

#[derive(Debug)]
pub struct UserId(uuid::Uuid);

impl UserId {
    pub fn new(id: uuid::Uuid) -> UserId {
        UserId(id)
    }

    pub fn get(&self) -> &uuid::Uuid {
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
