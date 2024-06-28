use crate::{
    api::{
        endpoints::{
            get_me::get_me_handler,
            healthcheck::healthcheck,
            login::login_handler,
            logout::logout_handler,
            register::{
                register_handler, AuthRepository, RegisterUserError, RegisterUserRequest,
                UserEmailEmptyError, UserPasswordEmptyError,
            },
        },
        middlewares::authentication::auth,
        utils::password_hasher,
    },
    helper::config::Config,
    model::user::{FilteredUser, User},
};
use anyhow::{anyhow, Context};
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use redis::Client;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

pub struct AppState<AR: AuthRepository> {
    pub auth_repository: AR,
    pub db: Pool<Postgres>,
    pub env: Config,
    pub redis: Client,
}

pub async fn run(listener: TcpListener, config: Config) {
    let pool = connect_to_database(&config).await;

    let redis_client = match Client::open(config.redis_url.to_owned()) {
        Ok(client) => {
            println!("Connection to redis successful");
            client
        }
        Err(err) => {
            println!("Failed to connect to redis with error: {}", err);
            std::process::exit(1);
        }
    };

    let postgres = PostgresDB::new(&config.database_url).await.unwrap();

    let app_state = Arc::new(AppState {
        auth_repository: postgres,
        db: pool,
        env: config,
        redis: redis_client,
    });

    let app = app(app_state);
    axum::serve(listener, app).await.unwrap();
}

fn app<AR: AuthRepository>(app_state: Arc<AppState<AR>>) -> Router {
    Router::new()
        .route("/api/healthcheck", get(healthcheck))
        .route("/api/register", post(register_handler))
        .route("/api/login", post(login_handler))
        .route(
            "/api/logout",
            get(logout_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/users/me",
            get(get_me_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(app_state)
}

pub async fn connect_to_database(config: &Config) -> Pool<Postgres> {
    match PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => {
            println!("Enstablished db connection");
            pool
        }
        Err(err) => {
            println!("Failed to connecto to db with error: {:?}", err);
            std::process::exit(1)
        }
    }
}

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
        request: &crate::api::endpoints::register::RegisterUserRequest,
    ) -> Result<crate::model::user::FilteredUser, crate::api::endpoints::register::RegisterUserError>
    {
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
        Ok(FilteredUser::from(user))
    }
}

impl PostgresDB {
    async fn is_unique_constrain_violation(
        &self,
        request: &RegisterUserRequest,
    ) -> anyhow::Result<()> {
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
                }
                .into());
            } else {
                return Ok(());
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum ApiError {
    InternalServerError(String),
    UnprocessableEntity(String),
}

impl From<RegisterUserError> for ApiError {
    fn from(value: RegisterUserError) -> Self {
        match value {
            RegisterUserError::Duplicate { email } => {
                Self::UnprocessableEntity(format!("User with email {} already exists", email))
            }
            RegisterUserError::Unknown(cause) => {
                tracing::error!("{:?}\n{}", cause, cause.backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

impl From<UserEmailEmptyError> for ApiError {
    fn from(_: UserEmailEmptyError) -> Self {
        Self::UnprocessableEntity("Email cannot be empty".to_string())
    }
}

impl From<UserPasswordEmptyError> for ApiError {
    fn from(_: UserPasswordEmptyError) -> Self {
        Self::UnprocessableEntity("Password cannot be empty".to_string())
    }
}
