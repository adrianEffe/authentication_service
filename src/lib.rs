pub mod api;

use api::healthcheck::healthcheck;
use axum::{routing::get, Router};

pub fn app() -> Router {
    Router::new().route("/api/healthcheck", get(healthcheck))
}
