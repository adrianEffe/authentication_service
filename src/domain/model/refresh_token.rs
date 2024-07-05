use thiserror::Error;

#[derive(Debug)]
pub struct RefreshRequest {
    token: Token,
}

impl RefreshRequest {
    fn new(token: String) -> RefreshRequest {
        RefreshRequest {
            token: Token(token),
        }
    }

    fn get_token(&self) -> &str {
        self.token.0.as_str()
    }
}

#[derive(Debug)]
struct Token(String);

#[derive(Debug)]
pub struct RefreshResponse;

#[derive(Debug, Error)]
pub enum RefreshTokenError {
    #[error("Token not valid")]
    InvalidCredentials,
    #[error("Refresh token not found")]
    MissingCredentials,
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
