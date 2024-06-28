use crate::{
    api::{
        endpoints::{
            get_me::get_me_handler,
            healthcheck::healthcheck,
            login::login_handler,
            logout::logout_handler,
            register::{register_handler, AuthRepository},
        },
        middlewares::authentication::auth,
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
    pub auth_repository: Arc<AR>,
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

    let postgres = PostgresPool::new(&config.database_url).await.unwrap();

    let app_state = Arc::new(AppState {
        auth_repository: Arc::new(postgres),
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
pub struct PostgresPool {
    pool: sqlx::Pool<Postgres>,
}

impl PostgresPool {
    pub async fn new(url: &str) -> anyhow::Result<PostgresPool> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(url)
            .await
            .with_context(|| format!("failed to open database url {url}"))?;

        Ok(PostgresPool { pool })
    }
}

impl AuthRepository for PostgresPool {
    async fn register(
        &self,
        request: &crate::api::endpoints::register::RegisterUserRequest,
    ) -> Result<crate::model::user::FilteredUser, crate::api::endpoints::register::RegisterUserError>
    {
        // TODO: - check if user exists from register:: module;
        let user = sqlx::query_as!(
            User,
            "INSERT INTO users (email, password) VALUES ($1, $2) RETURNING *",
            request.email.to_string().to_ascii_lowercase(),
            "hashed_password" // TODO: hashed password
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
