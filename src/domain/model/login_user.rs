use super::{user_email::UserEmail, user_password::UserPassword};
use thiserror::Error;

#[derive(Debug)]
pub struct LoginUserRequest {
    pub email: UserEmail,
    pub password: UserPassword,
}

impl LoginUserRequest {
    pub fn new(email: UserEmail, password: UserPassword) -> LoginUserRequest {
        LoginUserRequest { email, password }
    }
}

#[derive(Debug, Error)]
pub enum LoginUserError {
    #[error("Invalid email or password")]
    InvalidCredentials,
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
