use crate::domain::model::{
    auth::{AuthRequest, AuthorizationError},
    auth_middleware::AuthMiddleware,
    login_response::LoginResponse,
    login_user::{LoginUserError, LoginUserRequest},
    logout::{LogoutRequest, LogoutResponse},
    refresh_token::{RefreshRequest, RefreshResponse, RefreshTokenError},
    register_user::{RegisterUserError, RegisterUserRequest},
    user::FilteredUser,
};

use std::future::Future;

/// Trait representing authentication services in the application.
///
/// The `AuthService` trait defines the necessary methods for user registration, login,
/// authentication, logout, and token refreshing. Implementations of this trait
/// provide the actual logic for handling these operations, which can involve interactions
/// with databases, caches, and other services.
///
/// # Implementors
///
/// Any struct that implements the `AuthService` trait must be `Send`, `Sync`, and `'static`.
/// This ensures that instances of the implementing struct can be safely shared across
/// threads and have a static lifetime.
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
