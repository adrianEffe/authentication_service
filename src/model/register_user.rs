use super::{user_email::UserEmail, user_password::UserPassword};
use thiserror::Error;

#[derive(Debug)]
pub struct RegisterUserRequest {
    pub email: UserEmail,
    pub password: UserPassword,
}

impl RegisterUserRequest {
    pub fn new(email: UserEmail, password: UserPassword) -> Self {
        RegisterUserRequest { email, password }
    }
}

#[derive(Debug, Error)]
pub enum RegisterUserError {
    #[error("user with email {email} already exists")]
    Duplicate { email: UserEmail },
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
