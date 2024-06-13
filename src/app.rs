use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub struct AppState {
    db: Pool<Postgres>,
}

use crate::{api::healthcheck::healthcheck, helper::config::Config};
use axum::{routing::get, Router};

use std::sync::Arc;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub async fn app(config: Config) -> Router {
    let pool = match PgPoolOptions::new()
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
    };

    let app_state = Arc::new(AppState { db: pool });

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
