use core::fmt::Display;
use thiserror::Error;

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
