use crate::api::endpoints::refresh::refresh_access_token_handler;
use crate::domain::auth_service::AuthService;
use crate::repositories::auth_repository::PostgresDB;
use crate::repositories::cache_repository::RedisCache;
use crate::service::auth_service::Service;
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

pub struct AppState<AS: AuthService> {
    pub auth_service: AS,
}

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
