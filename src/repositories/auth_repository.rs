use crate::domain::{
    model::{
        auth::AuthorizationError,
        login_user::{LoginUserError, LoginUserRequest},
        register_user::{RegisterUserError, RegisterUserRequest},
        user::{FilteredUser, User},
        user_email::UserEmail,
        user_id::UserId,
    },
    repositories::auth_repository::AuthRepository,
};
use anyhow::{anyhow, Context};
use sqlx::{postgres::PgPoolOptions, Postgres};

#[derive(Clone, Debug)]
pub struct PostgresDB {
    pool: sqlx::Pool<Postgres>,
}

impl PostgresDB {
    pub async fn new(url: &str) -> anyhow::Result<PostgresDB> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(url)
            .await
            .with_context(|| format!("failed to open database url {url}"))?;

        Ok(PostgresDB { pool })
    }
}

impl AuthRepository for PostgresDB {
    async fn register(
        &self,
        request: &RegisterUserRequest,
    ) -> Result<FilteredUser, RegisterUserError> {
        self.is_unique_constrain_violation(request).await?;

        let user = sqlx::query_as!(
            User,
            "INSERT INTO users (email, password) VALUES ($1, $2) RETURNING *",
            request.email.get().to_ascii_lowercase(),
            request.hashed_password.get(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            anyhow!(e).context(format!(
                "Database error while registering user with email {}",
                request.email
            ))
        })?;
        Ok(FilteredUser::from(&user))
    }

    async fn login(&self, request: &LoginUserRequest) -> Result<User, LoginUserError> {
        self.fetch_user_by_email(&request.email).await
    }

    async fn auth(&self, request: &UserId) -> Result<User, AuthorizationError> {
        self.fetch_user_by_id(request).await
    }
}

impl PostgresDB {
    async fn is_unique_constrain_violation(
        &self,
        request: &RegisterUserRequest,
    ) -> Result<(), RegisterUserError> {
        let user_exists: Option<bool> =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
                .bind(request.email.to_string().to_ascii_lowercase())
                .fetch_one(&self.pool)
                .await
                .map_err(|e| {
                    anyhow!(e).context(format!("Database error for: {}", request.email))
                })?;
        if let Some(exists) = user_exists {
            if exists {
                return Err(RegisterUserError::Duplicate {
                    email: request.email.clone(),
                });
            } else {
                return Ok(());
            }
        }
        Ok(())
    }

    async fn fetch_user_by_email(&self, email: &UserEmail) -> Result<User, LoginUserError> {
        sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email = $1",
            email.get().to_ascii_lowercase()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            anyhow!(e).context(format!("Database error while looking up email: {}", email))
        })?
        .ok_or_else(|| LoginUserError::InvalidCredentials)
    }

    async fn fetch_user_by_id(&self, user_id: &UserId) -> Result<User, AuthorizationError> {
        sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id.get())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                anyhow!(e).context(format!(
                    "Database error while looking up user id: {:?}",
                    user_id
                ))
            })?
            .ok_or_else(|| AuthorizationError::InvalidCredentials {
                reason: "The user belonging to this token no longer exists".to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use std::{mem, ops::DerefMut, sync::Arc};

    use anyhow::anyhow;
    use tokio::sync::Mutex;

    use crate::domain::{
        model::{
            auth::AuthorizationError,
            login_user::{LoginUserError, LoginUserRequest},
            register_user::{HashedUserPassword, RegisterUserError, RegisterUserRequest},
            user::{FilteredUser, User},
            user_email::UserEmail,
            user_id::UserId,
            user_password::UserPassword,
        },
        repositories::auth_repository::AuthRepository,
    };

    struct MockAuthRepository {
        /// It would be great for result to just take a Result instead of the below, unfortunately
        /// it needs to conform to `Clone` but RegisterUserError` has an `Unknown` variant that
        /// might wrap errors that are not Clone.
        register_result: Arc<Mutex<Result<FilteredUser, RegisterUserError>>>,
        auth_result: Arc<Mutex<Result<User, AuthorizationError>>>,
        login_result: Arc<Mutex<Result<User, LoginUserError>>>,
    }

    impl AuthRepository for MockAuthRepository {
        async fn register(
            &self,
            _request: &RegisterUserRequest,
        ) -> Result<FilteredUser, RegisterUserError> {
            let mut guard = self.register_result.lock().await;
            let mut result = Err(RegisterUserError::Unknown(anyhow!("substitute error")));
            mem::swap(guard.deref_mut(), &mut result);
            result
        }

        async fn auth(&self, _request: &UserId) -> Result<User, AuthorizationError> {
            let mut guard = self.auth_result.lock().await;
            let mut result = Err(AuthorizationError::Unknown(anyhow!("substitute error")));
            mem::swap(guard.deref_mut(), &mut result);
            result
        }

        async fn login(&self, _request: &LoginUserRequest) -> Result<User, LoginUserError> {
            let mut guard = self.login_result.lock().await;
            let mut result = Err(LoginUserError::Unknown(anyhow!("substitute error")));
            mem::swap(guard.deref_mut(), &mut result);
            result
        }
    }

    // TODO: - Remove sanity check tests
    #[tokio::test]
    async fn test_register_success() {
        let email = "adrian@email.com";
        let password = "password";
        let user = User::new(email, password);
        let filtered_user = FilteredUser::from(&user);
        let register_result = Arc::new(Mutex::new(Ok(filtered_user)));
        let auth_result = Arc::new(Mutex::new(Ok(user.clone())));
        let login_result = Arc::new(Mutex::new(Ok(user)));

        let mock_repo = MockAuthRepository {
            register_result,
            auth_result,
            login_result,
        };

        let register = mock_repo
            .register(&RegisterUserRequest::new(
                UserEmail::new(email).unwrap(),
                HashedUserPassword::new(UserPassword::new(password).unwrap()).unwrap(),
            ))
            .await;

        assert_eq!(email.to_string(), register.unwrap().email);
    }

    #[tokio::test]
    async fn test_auth_success() {
        let email = "adrian@email.com";
        let password = "password";
        let mut user = User::new(email, password);
        let uuid = uuid::Uuid::new_v4();
        user.id = uuid;
        let filtered_user = FilteredUser::from(&user);
        let register_result = Arc::new(Mutex::new(Ok(filtered_user)));
        let auth_result = Arc::new(Mutex::new(Ok(user.clone())));
        let login_result = Arc::new(Mutex::new(Ok(user)));

        let user_id = UserId::new(uuid);

        let mock_repo = MockAuthRepository {
            register_result,
            auth_result,
            login_result,
        };

        let register = mock_repo.auth(&user_id).await;

        assert_eq!(user_id.get(), &register.unwrap().id);
    }
}
