use crate::domain::model::{
    auth::{AuthRequest, AuthorizationError},
    auth_middleware::AuthMiddleware,
    login_response::LoginResponse,
    login_user::{LoginUserError, LoginUserRequest},
    logout::{LogoutRequest, LogoutResponse},
    register_user::{RegisterUserError, RegisterUserRequest},
    user::FilteredUser,
};

use std::future::Future;

use super::model::refresh_token::{RefreshRequest, RefreshResponse, RefreshTokenError};

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

    fn logout(
        &self,
        request: &LogoutRequest,
    ) -> impl Future<Output = Result<LogoutResponse, AuthorizationError>> + Send;

    fn refresh(
        &self,
        request: &RefreshRequest,
    ) -> impl Future<Output = Result<RefreshResponse, RefreshTokenError>> + Send;
}
