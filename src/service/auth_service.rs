use anyhow::anyhow;

use crate::{
    api::utils::{
        jwt::{generate_jwt, verify_jwt},
        password_hasher::is_valid,
    },
    domain::{
        auth_service::AuthService,
        model::{
            auth::{AuthRequest, AuthorizationError},
            auth_middleware::AuthMiddleware,
            login_response::LoginResponse,
            login_user::{LoginUserError, LoginUserRequest},
            logout::{LogoutRequest, LogoutResponse},
            register_user::{RegisterUserError, RegisterUserRequest},
            user::FilteredUser,
            user_id::UserId,
        },
        repositories::{auth_repository::AuthRepository, cache_repository::CacheRepository},
    },
    helper::config::Config,
};

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
        self.repo.register(request).await
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

        self.cache
            .save_token_data(&access_token_details, self.config.access_token_max_age)
            .await
            .map_err(|e| anyhow!(e).context("Failed redis operation"))?;

        let access_token = access_token_details
            .token
            .ok_or_else(|| anyhow!("Failed to generate token"))?;

        Ok(LoginResponse {
            access_token,
            access_token_max_age: self.config.access_token_max_age,
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
            .auth(&UserId::new(access_token_details.user_id))
            .await?;

        Ok(AuthMiddleware::new(user, access_token_details.token_uuid))
    }

    async fn logout(&self, request: &LogoutRequest) -> Result<LogoutResponse, AuthorizationError> {
        self.cache.delete_token(request.get_uuid()).await?;
        Ok(LogoutResponse::new("User logged out"))
    }
}
