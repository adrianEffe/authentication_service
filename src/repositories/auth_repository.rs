use crate::domain::repositories::auth_repository::AuthRepository;
use crate::model::auth::{AuthRequest, AuthorizationError, UserId};
use crate::model::login_user::{LoginUserError, LoginUserRequest};
use crate::model::register_user::{RegisterUserError, RegisterUserRequest};
use crate::model::user::{FilteredUser, User};
use crate::model::user_email::UserEmail;
use anyhow::{anyhow, Context};
use sqlx::{postgres::PgPoolOptions, Postgres};

#[derive(Debug)]
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

    async fn auth(&self, request: &AuthRequest) -> Result<User, AuthorizationError> {
        self.fetch_user_by_id(&request.user_id).await
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
            .ok_or_else(|| AuthorizationError::InvalidCredentials)
    }
}
