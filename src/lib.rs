pub mod api;

use api::healthcheck::healthcheck;
use axum::{routing::get, Router};

use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn app() -> Router {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    Router::new().route(
        "/api/healthcheck",
        get(healthcheck).layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        ),
    )
}
