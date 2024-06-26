use crate::model::auth::{AuthRequest, AuthorizationError};
use crate::model::login_user::{LoginUserError, LoginUserRequest};
use crate::model::register_user::{RegisterUserError, RegisterUserRequest};
use crate::model::user::{FilteredUser, User};
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

    fn auth(
        &self,
        request: &AuthRequest,
    ) -> impl Future<Output = Result<User, AuthorizationError>> + Send;
}
