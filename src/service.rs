use crate::{
    domain::{
        auth_service::AuthService,
        repositories::{auth_repository::AuthRepository, cache_repository::CacheRepository},
    },
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
        todo!();
    }

    async fn auth(&self, request: &AuthRequest) -> Result<User, AuthorizationError> {
        todo!();
    }
}
