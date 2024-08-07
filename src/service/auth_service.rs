use anyhow::anyhow;

use crate::{
    api::utils::{
        jwt::{generate_jwt, verify_jwt},
        security::is_valid,
    },
    domain::{
        auth_service::AuthService,
        model::{
            auth::{AuthRequest, AuthorizationError},
            auth_middleware::AuthMiddleware,
            login_response::LoginResponse,
            login_user::{LoginUserError, LoginUserRequest},
            logout::{LogoutRequest, LogoutResponse},
            refresh_token::{RefreshRequest, RefreshResponse, RefreshTokenError},
            register_user::{RegisterUserError, RegisterUserRequest},
            token::CacheToken,
            user::FilteredUser,
            user_id::UserId,
        },
        repositories::{auth_repository::AuthRepository, cache_repository::CacheRepository},
    },
    helper::config::Config,
};

/// A service struct that implements the `AuthService` trait, providing authentication-related functionality.
///
/// The `Service` struct interacts with the authentication repository and cache repository to
/// handle registration, login, token validation, logout, and token refreshing. It uses the configuration
/// parameters provided by the `Config` struct to manage tokens and other settings.
///
/// # Type Parameters
///
/// * `R` - A type that implements the `AuthRepository` trait, providing methods to interact with the authentication database.
/// * `C` - A type that implements the `CacheRepository` trait, providing methods to interact with the cache storage.
#[derive(Debug)]
pub struct Service<R, C>
where
    R: AuthRepository,
    C: CacheRepository,
{
    pub repo: R,
    pub cache: C,
    pub config: Config,
}

impl<R, C> AuthService for Service<R, C>
where
    R: AuthRepository,
    C: CacheRepository,
{
    async fn register(
        &self,
        request: &RegisterUserRequest,
    ) -> Result<FilteredUser, RegisterUserError> {
        self.repo
            .register(request)
            .await
            .map_err(RegisterUserError::from)
    }

    async fn login(&self, request: &LoginUserRequest) -> Result<LoginResponse, LoginUserError> {
        let user = self.repo.login(request).await?;

        let is_valid = is_valid(request.password.get(), &user.password);
        if !is_valid {
            return Err(LoginUserError::InvalidCredentials);
        }

        let access_token_details = generate_jwt(
            user.id,
            self.config.access_token_max_age,
            &self.config.access_token_private_key,
        )?;

        let refresh_token_details = generate_jwt(
            user.id,
            self.config.refresh_token_max_age,
            &self.config.refresh_token_private_key,
        )?;

        self.cache
            .save_tokens_data(
                &CacheToken::new(
                    access_token_details.token_uuid,
                    access_token_details.user_id,
                    self.config.access_token_max_age,
                ),
                &CacheToken::new(
                    refresh_token_details.token_uuid,
                    refresh_token_details.user_id,
                    self.config.refresh_token_max_age,
                ),
            )
            .await
            .map_err(|e| anyhow!(e).context("Failed redis operation while saving tokens"))?;

        let access_token = access_token_details
            .token
            .ok_or_else(|| anyhow!("Failed to generate access token"))?;

        let refresh_token = refresh_token_details
            .token
            .ok_or_else(|| anyhow!("Failed to generate refresh token"))?;

        Ok(LoginResponse {
            access_token,
            access_token_max_age: self.config.access_token_max_age,
            refresh_token,
            refresh_token_max_age: self.config.refresh_token_max_age,
        })
    }

    async fn auth(&self, request: &AuthRequest) -> Result<AuthMiddleware, AuthorizationError> {
        let access_token_details = verify_jwt(
            &self.config.access_token_public_key,
            request.access_token.get(),
        )
        .map_err(|_| AuthorizationError::InvalidCredentials {
            reason: "Access token no longer valid".to_string(),
        })?;

        self.cache
            .verify_active_session(&access_token_details)
            .await
            .map_err(AuthorizationError::from)?;

        let user = self
            .repo
            .fetch_user_by_id(&UserId::new(access_token_details.user_id))
            .await?;

        Ok(AuthMiddleware::new(user, access_token_details.token_uuid))
    }

    async fn logout(&self, request: &LogoutRequest) -> Result<LogoutResponse, AuthorizationError> {
        self.cache.delete_token(request.get_uuid()).await?;
        Ok(LogoutResponse::new("User logged out"))
    }

    async fn refresh(
        &self,
        request: &RefreshRequest,
    ) -> Result<RefreshResponse, RefreshTokenError> {
        let refresh_token_details =
            verify_jwt(&self.config.refresh_token_public_key, request.get_token()).map_err(
                |_| RefreshTokenError::InvalidCredentials {
                    reason: "Refresh token no longer valid".to_string(),
                },
            )?;

        self.cache
            .verify_active_session(&refresh_token_details)
            .await
            .map_err(RefreshTokenError::from)?;

        let user = self
            .repo
            .fetch_user_by_id(&UserId::new(refresh_token_details.user_id))
            .await?;

        let access_token_details = generate_jwt(
            user.id,
            self.config.access_token_max_age,
            &self.config.access_token_private_key,
        )?;

        self.cache
            .save_token_data(&CacheToken::new(
                access_token_details.token_uuid,
                access_token_details.user_id,
                self.config.access_token_max_age,
            ))
            .await?;

        let access_token = access_token_details
            .token
            .ok_or_else(|| anyhow!("Failed to generate access token"))?;

        Ok(RefreshResponse {
            access_token,
            access_token_max_age: self.config.access_token_max_age,
        })
    }
}
