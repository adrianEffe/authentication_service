use crate::api::utils::password_hasher;
use crate::domain::repositories::auth_repository::AuthRepository;
use crate::model::login_response::LoginResponse;
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

        let hashed_password = password_hasher::hash_password(request.password.get())?;

        let user = sqlx::query_as!(
            User,
            "INSERT INTO users (email, password) VALUES ($1, $2) RETURNING *",
            request.email.get().to_ascii_lowercase(),
            hashed_password,
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

    async fn login(&self, request: &LoginUserRequest) -> Result<LoginResponse, LoginUserError> {
        todo!();
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

    async fn fetch_user(&self, email: &UserEmail) -> Result<User, LoginUserError> {
        sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email = $1",
            email.to_string().to_ascii_lowercase()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            anyhow!(e).context(format!("Database error while looking up email: {}", email))
        })?
        .ok_or_else(|| LoginUserError::InvalidCredentials)
    }
}
