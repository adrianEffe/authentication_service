use crate::domain::repositories::auth_repository::AuthRepository;
use crate::model::api_error::ApiError;
use crate::repositories::auth_repository::PostgresDB;
use crate::{
    api::{
        endpoints::{
            get_me::get_me_handler, healthcheck::healthcheck, login::login_handler,
            logout::logout_handler, register::register_handler,
        },
        middlewares::authentication::auth,
        utils::status::Status,
    },
    helper::config::Config,
};
use axum::{http::StatusCode, response::Response, Json};
use axum::{
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use redis::Client;
use serde::Serialize;
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

// TODO: move this

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    status: Status,
    data: Option<T>,
    message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        ApiResponse {
            status: Status::Success,
            data: Some(data),
            message: None,
        }
    }

    pub fn error(error: ApiError) -> Self {
        ApiResponse {
            status: Status::Failure,
            data: None,
            message: Some(error.to_string()),
        }
    }
}

// TODO: - make this better so you can pass proper status codes
impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status_code = match self.status {
            Status::Success => StatusCode::OK,
            Status::Failure => StatusCode::BAD_REQUEST,
        };
        (status_code, Json(self)).into_response()
    }
}
