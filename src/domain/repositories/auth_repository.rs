use crate::domain::model::{
    auth_repo_errors::AuthRepositoryError,
    login_user::LoginUserRequest,
    register_user::RegisterUserRequest,
    user::{FilteredUser, User},
    user_id::UserId,
};
use std::future::Future;

/// Trait defining the contract for authentication-related database repository operations.
///
/// The `AuthRepository` trait specifies the necessary methods for user registration,
/// login, and fetching user details by ID. Implementing this trait allows for
/// interaction with various data storage backends.
///
/// # Requirements
///
/// Any struct that implements the `AuthRepository` trait must be `Send`, `Sync`,
/// and have a `'static` lifetime. This ensures that instances of the implementing
/// struct can be safely shared across threads and have a static lifetime.
///
/// # Errors
///
/// The methods in this trait return a `Result` with the associated data type on success
/// or an `AuthRepositoryError` on failure.
pub trait AuthRepository: Send + Sync + 'static {
    fn register(
        &self,
        request: &RegisterUserRequest,
    ) -> impl Future<Output = Result<FilteredUser, AuthRepositoryError>> + Send;

    fn login(
        &self,
        request: &LoginUserRequest,
    ) -> impl Future<Output = Result<User, AuthRepositoryError>> + Send;

    fn fetch_user_by_id(
        &self,
        request: &UserId,
    ) -> impl Future<Output = Result<User, AuthRepositoryError>> + Send;
}
