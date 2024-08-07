use crate::domain::{
    model::{
        auth_repo_errors::AuthRepositoryError,
        login_user::LoginUserRequest,
        register_user::RegisterUserRequest,
        user::{FilteredUser, User},
        user_email::UserEmail,
        user_id::UserId,
    },
    repositories::auth_repository::AuthRepository,
};
use anyhow::Context;
use sqlx::{postgres::PgPoolOptions, Postgres};

#[derive(Clone, Debug)]
pub struct PostgresDB {
    pool: sqlx::Pool<Postgres>,
}

// A PostgreSQL-based implementation of the `AuthRepository` trait.
///
/// The `PostgresDB` struct provides methods for user registration, login,
/// and fetching user details using a PostgreSQL database.
///
/// # Fields
///
/// * `pool` - The connection pool to the PostgreSQL database.
///
/// # Errors
///
/// Methods in this implementation return an `AuthRepositoryError` on failure,
/// which includes details about the specific error encountered.
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
    ) -> Result<FilteredUser, AuthRepositoryError> {
        self.is_unique_constrain_violation(request).await?;

        let user = sqlx::query_as!(
            User,
            "INSERT INTO users (email, password) VALUES ($1, $2) RETURNING *",
            request.email.get().to_ascii_lowercase(),
            request.hashed_password.get(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AuthRepositoryError::Database {
            reason: format!(
                "Database error while registering user with email {}: {}",
                request.email, e
            ),
        })?;

        Ok(FilteredUser::from(&user))
    }

    async fn login(&self, request: &LoginUserRequest) -> Result<User, AuthRepositoryError> {
        self.fetch_user_by_email(&request.email).await
    }

    async fn fetch_user_by_id(&self, request: &UserId) -> Result<User, AuthRepositoryError> {
        self.fetch_user_by_id(request).await
    }
}

impl PostgresDB {
    /// Checks if a user with the given email already exists in the database.
    ///
    /// This method queries the database to determine whether a user with the specified
    /// email address already exists, which helps in enforcing a unique constraint on
    /// user registration. If a user with the given email exists, an `AuthRepositoryError::Duplicate`
    /// error is returned. Otherwise, the method returns `Ok(())`.
    ///
    /// # Arguments
    ///
    /// * `request` - A reference to the `RegisterUserRequest` containing the user registration details.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// * `Ok(())` if the email is unique and no user exists with the given email.
    /// * `Err(AuthRepositoryError::Duplicate)` if a user with the given email already exists.
    /// * `Err(AuthRepositoryError::Database)` if there is a database error during the check.
    ///
    /// # Errors
    ///
    /// This method returns an `AuthRepositoryError` in the following scenarios:
    /// * `AuthRepositoryError::Duplicate` if a user with the specified email already exists.
    /// * `AuthRepositoryError::Database` if there is an error querying the database.
    async fn is_unique_constrain_violation(
        &self,
        request: &RegisterUserRequest,
    ) -> Result<(), AuthRepositoryError> {
        let user_exists: Option<bool> =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
                .bind(request.email.to_string().to_ascii_lowercase())
                .fetch_one(&self.pool)
                .await
                .map_err(|e| AuthRepositoryError::Database {
                    reason: format!(
                        "Database error while checking if user with email {} exists: {}",
                        request.email, e
                    ),
                })?;

        if let Some(exists) = user_exists {
            if exists {
                return Err(AuthRepositoryError::Duplicate {
                    email: request.email.clone(),
                });
            } else {
                return Ok(());
            }
        }
        Ok(())
    }

    async fn fetch_user_by_email(&self, email: &UserEmail) -> Result<User, AuthRepositoryError> {
        sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email = $1",
            email.get().to_ascii_lowercase()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthRepositoryError::Database {
            reason: format!("Database error: {}", e),
        })?
        .ok_or_else(|| AuthRepositoryError::InvalidCredentials {
            reason: "User does not exist".to_string(),
        })
    }

    async fn fetch_user_by_id(&self, user_id: &UserId) -> Result<User, AuthRepositoryError> {
        sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id.get())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AuthRepositoryError::Database {
                reason: format!(
                    "Database error while looking up user id {:?}: {}",
                    user_id, e
                ),
            })?
            .ok_or_else(|| AuthRepositoryError::InvalidCredentials {
                reason: "The user belonging to this token no longer exists".to_string(),
            })
    }
}
