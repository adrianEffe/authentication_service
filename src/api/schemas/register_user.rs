use crate::api::endpoints::register::{RegisterUserRequest, UserEmail, UserPassword};
use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RegisterUserSchema {
    pub email: String,
    pub password: String,
}

impl RegisterUserSchema {
    fn try_into_domain(self) -> Result<RegisterUserRequest> {
        let email = UserEmail::new(&self.email)?;
        let password = UserPassword::new(&self.password)?;
        Ok(RegisterUserRequest::new(email, password))
    }
}
