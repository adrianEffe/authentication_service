use crate::domain::model::{
    auth::AuthorizationError,
    login_user::{LoginUserError, LoginUserRequest},
    register_user::{RegisterUserError, RegisterUserRequest},
    user::{FilteredUser, User},
    user_id::UserId,
};
use std::future::Future;

pub trait AuthRepository: Send + Sync + 'static {
    fn register(
        &self,
        request: &RegisterUserRequest,
    ) -> impl Future<Output = Result<FilteredUser, RegisterUserError>> + Send;

    fn login(
        &self,
        request: &LoginUserRequest,
    ) -> impl Future<Output = Result<User, LoginUserError>> + Send;

    fn fetch_user_by_id(
        &self,
        request: &UserId,
    ) -> impl Future<Output = Result<User, AuthorizationError>> + Send;
}
