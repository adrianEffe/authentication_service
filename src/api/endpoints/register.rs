use crate::application::{ApiError, ApiResponse, AppState};
use crate::model::user::{FilteredUser, User};
use crate::{
    api::{
        schemas::register_user::RegisterUserSchema,
        utils::{
            password_hasher,
            status::{response_data, response_message, Status},
        },
    },
    model::user::UserResponse,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use sqlx::{Pool, Postgres};
use std::fmt::Display;
use std::future::Future;
use std::sync::Arc;
use thiserror::Error;

pub trait AuthRepository: Send + Sync + 'static {
    fn register(
        &self,
        request: &RegisterUserRequest,
    ) -> impl Future<Output = Result<FilteredUser, RegisterUserError>> + Send;
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

pub async fn register_handler<AR: AuthRepository>(
    State(state): State<Arc<AppState<AR>>>,
    Json(body): Json<RegisterUserSchema>,
) -> Result<ApiResponse<FilteredUser>, ApiError> {
    let domain_request = body.try_into_domain()?;

    state
        .auth_repository
        .register(&domain_request)
        .await
        .map_err(ApiError::from)
        .map(ApiResponse::success)
}
