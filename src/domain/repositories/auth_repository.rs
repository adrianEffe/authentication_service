use crate::domain::model::{
    auth_repo_errors::AuthRepositoryError,
    login_user::LoginUserRequest,
    register_user::RegisterUserRequest,
    user::{FilteredUser, User},
    user_id::UserId,
};
use std::future::Future;

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
