use serde::Deserialize;

use crate::model::{
    api_error::ApiError, login_user::LoginUserRequest, user_email::UserEmail,
    user_password::UserPassword,
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
