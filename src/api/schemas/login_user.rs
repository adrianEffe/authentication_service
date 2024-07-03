use serde::Deserialize;

use crate::{
    domain::model::{
        login_user::LoginUserRequest, user_email::UserEmail, user_password::UserPassword,
    },
    model::api_error::ApiError,
};

#[derive(Debug, Deserialize)]
pub struct LoginUserSchema {
    pub email: String,
    pub password: String,
}

impl LoginUserSchema {
    pub fn try_into_domain(&self) -> Result<LoginUserRequest, ApiError> {
        let email = UserEmail::new(&self.email)?;
        let password = UserPassword::new(&self.password)?;
        Ok(LoginUserRequest::new(email, password))
    }
}
