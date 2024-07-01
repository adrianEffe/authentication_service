use super::{user_email::UserEmail, user_password::UserPassword};
use thiserror::Error;

#[derive(Debug)]
struct LoginUserRequest {
    pub email: UserEmail,
    pub password: UserPassword,
}

#[derive(Debug, Error)]
pub enum LoginUserError {
    #[error("Invalid email or password")]
    InvalidCredentials,
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
