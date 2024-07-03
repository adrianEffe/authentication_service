use crate::{
    domain::model::{
        register_user::{HashedUserPassword, RegisterUserRequest},
        user_email::UserEmail,
        user_password::UserPassword,
    },
    model::api_error::ApiError,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RegisterUserSchema {
    pub email: String,
    pub password: String,
}

impl RegisterUserSchema {
    pub fn try_into_domain(self) -> Result<RegisterUserRequest, ApiError> {
        let email = UserEmail::new(&self.email)?;
        let password = UserPassword::new(&self.password)?;
        let hashed_password = HashedUserPassword::new(password)?;
        Ok(RegisterUserRequest::new(email, hashed_password))
    }
}
