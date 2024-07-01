use crate::model::{
    api_error::ApiError,
    register_user::{RegisterUserRequest, UserEmail, UserPassword},
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
        Ok(RegisterUserRequest::new(email, password))
    }
}
