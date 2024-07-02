use crate::model::auth::{AuthRequest, AuthorizationError};
use crate::model::auth_middleware::AuthMiddleware;
use crate::model::login_response::LoginResponse;
use crate::model::login_user::{LoginUserError, LoginUserRequest};
use crate::model::register_user::{RegisterUserError, RegisterUserRequest};
use crate::model::user::FilteredUser;
use std::future::Future;

pub trait AuthService: Send + Sync + 'static {
    fn register(
        &self,
        request: &RegisterUserRequest,
    ) -> impl Future<Output = Result<FilteredUser, RegisterUserError>> + Send;

    fn login(
        &self,
        request: &LoginUserRequest,
    ) -> impl Future<Output = Result<LoginResponse, LoginUserError>> + Send;

    fn auth(
        &self,
        request: &AuthRequest,
    ) -> impl Future<Output = Result<AuthMiddleware, AuthorizationError>> + Send;
}