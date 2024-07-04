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

#[cfg(test)]
mod test {
    use crate::{
        api::utils::{jwt::generate_jwt, password_hasher::hash_password},
        domain::{
            auth_service::AuthService,
            model::{
                auth::AuthRequest,
                login_user::LoginUserRequest,
                logout::LogoutRequest,
                register_user::{HashedUserPassword, RegisterUserRequest},
                user_email::UserEmail,
                user_password::UserPassword,
            },
        },
        helper::config::Config,
        repositories::test_helpers::{
            mock_auth_repository::test_helpers::MockAuthRepository,
            mock_cache_repository::test_helpers::MockCacheRepository,
        },
        service::auth_service::Service,
    };

    #[tokio::test]
    async fn test_register_success() {
        let email = "adrian@email.com";
        let password = "password";

        let repo = MockAuthRepository::success(email, password);
        let cache = MockCacheRepository::success();
        let config = Config::init();

        let state = Service {
            repo,
            cache,
            config,
        };

        let result = state
            .register(&RegisterUserRequest::new(
                UserEmail::new(email).unwrap(),
                HashedUserPassword::new(UserPassword::new(password).unwrap()).unwrap(),
            ))
            .await;

        assert_eq!(result.unwrap().email, email)
    }

    #[tokio::test]
    async fn test_register_failure() {
        let email = "adrian@email.com";
        let password = "password";

        let repo = MockAuthRepository::failure();
        let cache = MockCacheRepository::failure();
        let config = Config::init();

        let state = Service {
            repo,
            cache,
            config,
        };

        let result = state
            .register(&RegisterUserRequest::new(
                UserEmail::new(email).unwrap(),
                HashedUserPassword::new(UserPassword::new(password).unwrap()).unwrap(),
            ))
            .await;

        assert!(result.is_err())
    }

    #[tokio::test]
    async fn test_login_success() {
        let email = "adrian@email.com";
        let password = "password";
        let hashed_password = hash_password(password).unwrap();

        let repo = MockAuthRepository::success(email, &hashed_password);
        let cache = MockCacheRepository::success();
        let config = Config::init();

        let state = Service {
            repo,
            cache,
            config,
        };

        let result = state
            .login(&LoginUserRequest::new(
                UserEmail::new(email).unwrap(),
                UserPassword::new(password).unwrap(),
            ))
            .await;

        assert!(result.is_ok())
    }

    #[tokio::test]
    async fn test_login_invalid_password_failure() {
        let email = "adrian@email.com";
        let bad_password = "password";

        let repo = MockAuthRepository::success(email, bad_password);
        let cache = MockCacheRepository::success();
        let config = Config::init();

        let state = Service {
            repo,
            cache,
            config,
        };

        let result = state
            .login(&LoginUserRequest::new(
                UserEmail::new(email).unwrap(),
                UserPassword::new(bad_password).unwrap(),
            ))
            .await;

        assert!(result.is_err())
    }

    #[tokio::test]
    async fn test_login_database_failure() {
        let email = "adrian@email.com";
        let password = "password";

        let repo = MockAuthRepository::failure();
        let cache = MockCacheRepository::success();
        let config = Config::init();

        let state = Service {
            repo,
            cache,
            config,
        };

        let result = state
            .login(&LoginUserRequest::new(
                UserEmail::new(email).unwrap(),
                UserPassword::new(password).unwrap(),
            ))
            .await;

        assert!(result.is_err())
    }

    #[tokio::test]
    async fn test_login_cache_failure() {
        let email = "adrian@email.com";
        let password = "password";

        let repo = MockAuthRepository::success(email, password);
        let cache = MockCacheRepository::failure();
        let config = Config::init();

        let state = Service {
            repo,
            cache,
            config,
        };

        let result = state
            .login(&LoginUserRequest::new(
                UserEmail::new(email).unwrap(),
                UserPassword::new(password).unwrap(),
            ))
            .await;

        assert!(result.is_err())
    }

    #[tokio::test]
    async fn test_auth_success() {
        let email = "adrian@email.com";
        let password = "password";
        let config = Config::init();

        let access_token_details = generate_jwt(
            uuid::Uuid::new_v4(),
            config.access_token_max_age,
            &config.access_token_private_key,
        )
        .unwrap();

        let repo = MockAuthRepository::success(email, password);
        let cache = MockCacheRepository::success();

        let state = Service {
            repo,
            cache,
            config,
        };

        let result = state
            .auth(&AuthRequest::new(access_token_details.token.unwrap()))
            .await;

        assert!(result.is_ok())
    }

    #[tokio::test]
    async fn test_invalid_token_failure() {
        let email = "adrian@email.com";
        let password = "password";

        let repo = MockAuthRepository::success(email, password);
        let cache = MockCacheRepository::success();
        let config = Config::init();

        let state = Service {
            repo,
            cache,
            config,
        };

        let result = state
            .auth(&AuthRequest::new("Invalid token".to_string()))
            .await;

        assert!(result.is_err())
    }

    #[tokio::test]
    async fn test_auth_db_failure() {
        let config = Config::init();

        let access_token_details = generate_jwt(
            uuid::Uuid::new_v4(),
            config.access_token_max_age,
            &config.access_token_private_key,
        )
        .unwrap();

        let repo = MockAuthRepository::failure();
        let cache = MockCacheRepository::success();

        let state = Service {
            repo,
            cache,
            config,
        };

        let result = state
            .auth(&AuthRequest::new(access_token_details.token.unwrap()))
            .await;

        assert!(result.is_err())
    }

    #[tokio::test]
    async fn test_auth_cache_failure() {
        let email = "adrian@email.com";
        let password = "password";
        let config = Config::init();

        let access_token_details = generate_jwt(
            uuid::Uuid::new_v4(),
            config.access_token_max_age,
            &config.access_token_private_key,
        )
        .unwrap();

        let repo = MockAuthRepository::success(email, password);
        let cache = MockCacheRepository::failure();

        let state = Service {
            repo,
            cache,
            config,
        };

        let result = state
            .auth(&AuthRequest::new(access_token_details.token.unwrap()))
            .await;

        assert!(result.is_err())
    }

    #[tokio::test]
    async fn test_logout_success() {
        let email = "adrian@email.com";
        let password = "password";
        let config = Config::init();

        let repo = MockAuthRepository::success(email, password);
        let cache = MockCacheRepository::success();

        let state = Service {
            repo,
            cache,
            config,
        };

        let result = state
            .logout(&LogoutRequest::new(uuid::Uuid::new_v4()))
            .await;

        assert!(result.is_ok())
    }

    #[tokio::test]
    async fn test_logout_cache_failure() {
        let email = "adrian@email.com";
        let password = "password";
        let config = Config::init();

        let repo = MockAuthRepository::success(email, password);
        let cache = MockCacheRepository::failure();

        let state = Service {
            repo,
            cache,
            config,
        };

        let result = state
            .logout(&LogoutRequest::new(uuid::Uuid::new_v4()))
            .await;

        assert!(result.is_err())
    }

    #[tokio::test]
    async fn test_logout_db_failure() {
        let config = Config::init();

        let repo = MockAuthRepository::failure();
        let cache = MockCacheRepository::success();

        let state = Service {
            repo,
            cache,
            config,
        };

        let result = state
            .logout(&LogoutRequest::new(uuid::Uuid::new_v4()))
            .await;

        assert!(result.is_ok())
    }
}
