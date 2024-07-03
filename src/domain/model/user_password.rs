use core::fmt::Display;
use thiserror::Error;

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
