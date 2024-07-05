use crate::api::utils::password_hasher;

use super::{
    auth_repo_errors::AuthRepositoryError, user_email::UserEmail, user_password::UserPassword,
};
use anyhow::{anyhow, Result};
use thiserror::Error;

#[derive(Debug)]
pub struct RegisterUserRequest {
    pub email: UserEmail,
    pub hashed_password: HashedUserPassword,
}

impl RegisterUserRequest {
    pub fn new(email: UserEmail, hashed_password: HashedUserPassword) -> Self {
        RegisterUserRequest {
            email,
            hashed_password,
        }
    }
}

#[derive(Debug, Error)]
pub enum RegisterUserError {
    #[error("user with email {email} already exists")]
    Duplicate { email: UserEmail },
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

impl From<AuthRepositoryError> for RegisterUserError {
    fn from(value: AuthRepositoryError) -> Self {
        match value {
            AuthRepositoryError::Duplicate { email } => RegisterUserError::Duplicate { email },
            _ => RegisterUserError::Unknown(anyhow!("Internal server error")),
        }
    }
}

#[derive(Debug)]
pub struct HashedUserPassword(String);

#[derive(Clone, Debug, Error)]
#[error("failed to hash password")]
pub struct PasswordHashingError;

impl HashedUserPassword {
    pub fn new(password: UserPassword) -> Result<HashedUserPassword, PasswordHashingError> {
        let hashed_password =
            password_hasher::hash_password(password.get()).map_err(|_| PasswordHashingError)?;
        Ok(HashedUserPassword(hashed_password))
    }

    pub fn get(&self) -> &str {
        &self.0
    }
}
