use crate::{
    api::{
        endpoints::{
            get_me::get_me_handler, healthcheck::healthcheck, login::login_handler,
            logout::logout_handler, refresh::refresh_access_token_handler,
            register::register_handler,
        },
        middlewares::authentication::auth,
    },
    domain::auth_service::AuthService,
    helper::config::Config,
    repositories::{auth_repository::PostgresDB, cache_repository::RedisCache},
    service::auth_service::Service,
};
use anyhow::Result;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

/// Holds the shared state for the application, including the authentication service.
///
/// The `AppState` struct is used to share state across different parts of the application.
/// It contains an instance of the authentication service, which is required by various
/// handlers for tasks such as user authentication and authorization.
///
/// # Type Parameters
///
/// * `AS` - A type that implements the `AuthService` trait. This allows the `AppState`
///   to be generic over any authentication service that conforms to the `AuthService` trait.
///
/// # Examples
///
/// ```rust
/// use std::sync::Arc;
/// use authentication_service::{application::AppState, domain::auth_service::AuthService};
///
/// // Assume we have some AuthService implementation
/// let auth_service = MyAuthService::new();
/// let app_state = Arc::new(AppState { auth_service });
/// ```
///
/// This example demonstrates how to create a new `AppState` with a concrete implementation
/// of the `AuthService` trait.
pub struct AppState<AS: AuthService> {
    pub auth_service: AS,
}

/// Asynchronously runs the application with the given TCP listener and configuration.
///
/// This function sets up the necessary components for the application, including
/// the PostgreSQL database connection and the Redis cache. It then initializes
/// the application state and starts the server using the Axum framework.
///
/// # Arguments
///
/// * `listener` - A `TcpListener` that listens for incoming TCP connections.
/// * `config` - A `Config` struct containing the configuration settings for the application.
///
/// # Returns
///
/// A `Result` which is:
/// - `Ok(())` if the server runs successfully.
/// - An error if there is an issue setting up the database connection, Redis cache,
///   or starting the server.
///
/// # Examples
///
/// ```rust,no_run
/// use std::net::TcpListener;
/// use authentication_service::{application::run, helper::config::Config};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let listener = TcpListener::bind("127.0.0.1:3000")?;
///     let config = Config::new();
///     run(listener, config).await?;
///     Ok(())
/// }
/// ```
///
/// # Errors
///
/// This function will return an error if:
/// - The PostgreSQL database connection cannot be established.
/// - The Redis cache cannot be initialized.
/// - The server fails to start.
pub async fn run(listener: TcpListener, config: Config) -> Result<()> {
    let postgres = PostgresDB::new(&config.database_url).await?;
    let redis = RedisCache::new(&config.redis_url);

    let service = Service {
        repo: postgres,
        cache: redis,
        config,
    };

    let app_state = Arc::new(AppState {
        auth_service: service,
    });

    let app = app(app_state);
    axum::serve(listener, app).await?;

    Ok(())
}

/// Creates a new Axum router with the given application state.
///
/// This function sets up the routes for the application and applies the necessary
/// middlewares and layers. It includes routes for health checks, authentication,
/// and user management. Each route is associated with its corresponding handler
/// function and middleware where required.
///
/// # Arguments
///
/// * `app_state` - An `Arc<AppState<AS>>` containing the shared application state.
///   The `AppState` includes the authentication service and any other shared state
///   needed by the handlers.
///
/// # Returns
///
/// A `Router` configured with the application's routes and middleware.
///
/// # Type Parameters
///
/// * `AS` - A type that implements the `AuthService` trait. This is used to abstract
///   over the authentication service implementation.
///
/// # Examples
///
/// ```rust
/// use std::sync::Arc;
/// use authentication_service::application::{app, AppState, AuthService};
///
/// // Assume we have some AuthService implementation
/// let auth_service = MyAuthService::new();
/// let app_state = Arc::new(AppState { auth_service });
/// let router = app(app_state);
/// ```
fn app<AS: AuthService>(app_state: Arc<AppState<AS>>) -> Router {
    Router::new()
        .route("/api/healthcheck", get(healthcheck))
        .route("/api/refresh", get(refresh_access_token_handler))
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
