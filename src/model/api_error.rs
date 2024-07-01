use crate::model::{
    register_user::RegisterUserError, user_email::UserEmailEmptyError,
    user_password::UserPasswordEmptyError,
};
use axum::{http::StatusCode, response::IntoResponse};

use super::{
    auth::AuthorizationError, login_user::LoginUserError, register_user::PasswordHashingError,
};

#[derive(Debug)]
pub enum ApiError {
    InternalServerError(String),
    UnprocessableEntity(String),
    Unauthorized(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::InternalServerError(msg) => write!(f, "{}", msg),
            ApiError::UnprocessableEntity(msg) => write!(f, "{}", msg),
            ApiError::Unauthorized(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<RegisterUserError> for ApiError {
    fn from(value: RegisterUserError) -> Self {
        match &value {
            RegisterUserError::Duplicate { email } => {
                Self::UnprocessableEntity(format!("User with email {} already exists", email))
            }
            RegisterUserError::Unknown(cause) => {
                tracing::error!("{:?}\n{}", cause, cause.backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

impl From<AuthorizationError> for ApiError {
    fn from(value: AuthorizationError) -> Self {
        match &value {
            AuthorizationError::InvalidCredentials { reason } => {
                Self::Unauthorized(reason.to_string())
            }
            AuthorizationError::Unknown(cause) => {
                tracing::error!("{:?}\n{}", cause, cause.backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

impl From<UserEmailEmptyError> for ApiError {
    fn from(_: UserEmailEmptyError) -> Self {
        Self::UnprocessableEntity("Email cannot be empty".to_string())
    }
}

impl From<UserPasswordEmptyError> for ApiError {
    fn from(_: UserPasswordEmptyError) -> Self {
        Self::UnprocessableEntity("Password cannot be empty".to_string())
    }
}

impl From<PasswordHashingError> for ApiError {
    fn from(_: PasswordHashingError) -> Self {
        Self::InternalServerError("Something went wrong".to_string())
    }
}

impl From<LoginUserError> for ApiError {
    fn from(value: LoginUserError) -> Self {
        match &value {
            LoginUserError::InvalidCredentials => {
                Self::Unauthorized("Invalid credentials".to_string())
            }
            LoginUserError::Unknown(cause) => {
                tracing::error!("{:?}\n{}", cause, cause.backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::UnprocessableEntity(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
        }
        .into_response()
    }
}
