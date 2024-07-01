use core::fmt::Display;
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

#[derive(Debug)]
pub struct UserPassword(String);

impl Display for UserPassword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Error)]
#[error("user email cannot be empty")]
pub struct UserPasswordEmptyError;

impl UserPassword {
    pub fn new(raw: &str) -> Result<Self, UserPasswordEmptyError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            Err(UserPasswordEmptyError)
        } else {
            Ok(Self(trimmed.to_string()))
        }
    }

    pub fn get(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct UserEmail(String);

#[derive(Clone, Debug, Error)]
#[error("user email cannot be empty")]
pub struct UserEmailEmptyError;

impl UserEmail {
    pub fn new(raw: &str) -> Result<Self, UserEmailEmptyError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            Err(UserEmailEmptyError)
        } else {
            Ok(Self(trimmed.to_string()))
        }
    }

    pub fn get(&self) -> &str {
        &self.0
    }
}

impl Display for UserEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
