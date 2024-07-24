use crate::api::utils::security;

use super::{
    auth_repo_errors::AuthRepositoryError, user_email::UserEmail, user_password::UserPassword,
};
use anyhow::{anyhow, Result};
use thiserror::Error;

#[derive(Debug)]
pub struct RegisterUserRequest {
    pub email: UserEmail,
    pub hashed_password: HashedUserPassword,
}

impl RegisterUserRequest {
    pub fn new(email: UserEmail, hashed_password: HashedUserPassword) -> Self {
        RegisterUserRequest {
            email,
            hashed_password,
        }
    }
}

#[derive(Debug, Error)]
pub enum RegisterUserError {
    #[error("user with email {email} already exists")]
    Duplicate { email: UserEmail },
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

impl From<AuthRepositoryError> for RegisterUserError {
    fn from(value: AuthRepositoryError) -> Self {
        match value {
            AuthRepositoryError::Duplicate { email } => RegisterUserError::Duplicate { email },
            _ => RegisterUserError::Unknown(anyhow!("Internal server error")),
        }
    }
}

#[derive(Debug)]
pub struct HashedUserPassword(String);

#[derive(Clone, Debug, Error)]
#[error("failed to hash password")]
pub struct PasswordHashingError;

/// Creates a new `HashedUserPassword` instance by hashing the provided user password.
///
/// This method takes a plain user password, hashes it using the `hash_password` function from the `security` module,
/// and then wraps the resulting hashed password in a `HashedUserPassword` instance.
///
/// The password is hashed using a secure hashing algorithm (such as Argon2) to ensure it is safely stored. The hashing
/// process adds a salt to the password to protect against rainbow table attacks and ensures that the password is securely
/// transformed into a hash suitable for storage.
///
/// # Arguments
///
/// * `password` - A `UserPassword` instance containing the plain text password that needs to be hashed.
///
/// # Returns
///
/// A `Result` containing either:
/// - `Ok(HashedUserPassword)` if the password is successfully hashed and wrapped in a `HashedUserPassword` instance.
/// - `Err(PasswordHashingError)` if there is an error during the hashing process. The error could be due to issues with
///   the hashing function or invalid input.
///
/// # Errors
///
/// This method returns a `PasswordHashingError` if the password hashing fails. Possible reasons for failure include
/// issues with the hashing algorithm or other unexpected errors during the hashing process.
///
/// # Examples
///
/// ```rust
/// use authentication_service::domain::model::user_password::UserPassword;
/// use authentication_service::domain::model::register_user::HashedUserPassword;
/// use authentication_service::domain::model::register_user::PasswordHashingError;
///
/// let password = UserPassword::new("my_secure_password").unwrap();
/// match HashedUserPassword::new(password) {
///     Ok(hashed_password) => println!("Password hashed successfully: {:?}", hashed_password),
///     Err(e) => eprintln!("Failed to hash password: {:?}", e),
/// }
/// ```
impl HashedUserPassword {
    pub fn new(password: UserPassword) -> Result<HashedUserPassword, PasswordHashingError> {
        let hashed_password =
            security::hash_password(password.get()).map_err(|_| PasswordHashingError)?;
        Ok(HashedUserPassword(hashed_password))
    }

    pub fn get(&self) -> &str {
        &self.0
    }
}
