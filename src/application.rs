use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub struct AppState {
    pub db: Pool<Postgres>,
}

use crate::{api::healthcheck::healthcheck, helper::config::Config};
use axum::{routing::get, Router};

use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub async fn run(listener: TcpListener, config: Config) {
    let pool = connect_to_database(&config).await;

    let app_state = Arc::new(AppState { db: pool });

    let app = app(app_state);
    axum::serve(listener, app).await.unwrap();
}

pub fn app(app_state: Arc<AppState>) -> Router {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    Router::new()
        .route(
            "/api/healthcheck",
            get(healthcheck).layer(
                TraceLayer::new_for_http()
                    .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                    .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
            ),
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
