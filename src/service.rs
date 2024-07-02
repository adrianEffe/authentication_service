use crate::{
    api::utils::{jwt::generate_jwt, password_hasher::is_valid},
    domain::{
        auth_service::AuthService,
        repositories::{auth_repository::AuthRepository, cache_repository::CacheRepository},
    },
    helper::{config::Config, redis_helper},
    model::{
        auth::{AuthRequest, AuthorizationError},
        login_user::{LoginUserError, LoginUserRequest},
        register_user::{RegisterUserError, RegisterUserRequest},
        user::{FilteredUser, User},
    },
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

    async fn login(&self, request: &LoginUserRequest) -> Result<User, LoginUserError> {
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
        // .map_err(|e| {
        //     ApiError::from(LoginUserError::Unknown(
        //         anyhow!(e).context("Failed to generate jwt token"),
        //     ))
        // })?;
        //
        // // TODO: - abstract redis away
        // redis_helper::save_token_data(
        //     &state,
        //     &access_token_details,
        //     state.env.access_token_max_age,
        // )
        // .await?;
        // .map_err(|e| {
        //     ApiError::from(LoginUserError::Unknown(
        //         anyhow!(e).context("Failed to save token to redis"),
        //     ))
        // })?;
        //
        // let access_token = access_token_details.token.ok_or_else(|| {
        //     ApiError::from(LoginUserError::Unknown(anyhow!("Failed to generate token")))
        // })?;
        //
        todo!();
    }

    async fn auth(&self, request: &AuthRequest) -> Result<User, AuthorizationError> {
        todo!();
    }
}
