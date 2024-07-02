use crate::domain::auth_service::AuthService;
use crate::domain::repositories::auth_repository::AuthRepository;
use crate::repositories::auth_repository::PostgresDB;
use crate::repositories::cache_repository::RedisCache;
use crate::service::Service;
use crate::{
    api::{
        endpoints::{
            get_me::get_me_handler, healthcheck::healthcheck, login::login_handler,
            logout::logout_handler, register::register_handler,
        },
        middlewares::authentication::auth,
    },
    helper::config::Config,
};
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use redis::Client;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

pub struct AppState<AR: AuthRepository, AS: AuthService> {
    pub auth_service: AS,
    pub auth_repository: AR,
    pub redis: Client,
}

pub async fn run(listener: TcpListener, config: Config) {
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

    let postgres = PostgresDB::new(&config.database_url).await.unwrap(); //TODO: handle unwrap
    let redis = RedisCache::new(&config.redis_url);

    let service = Service {
        repo: postgres.clone(), // TODO: remove clone
        cache: redis,
        config,
    };

    let app_state = Arc::new(AppState {
        auth_service: service,
        auth_repository: postgres,
        redis: redis_client,
    });

    let app = app(app_state);
    axum::serve(listener, app).await.unwrap(); //TODO: handle unwrap
}

fn app<AR: AuthRepository, AS: AuthService>(app_state: Arc<AppState<AR, AS>>) -> Router {
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
