use super::{
    auth_repo_errors::AuthRepositoryError, user_email::UserEmail, user_password::UserPassword,
};
use anyhow::anyhow;
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

impl From<AuthRepositoryError> for LoginUserError {
    fn from(value: AuthRepositoryError) -> Self {
        match value {
            AuthRepositoryError::InvalidCredentials { reason: _ } => {
                LoginUserError::InvalidCredentials
            }
            _ => LoginUserError::Unknown(anyhow!("Internal Server Error".to_string())),
        }
    }
}
