mod utils;
use axum::{response::IntoResponse, routing::get, Json, Router};
use utils::status::{response_message, Status};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/api/healthcheck", get(healthcheck));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn healthcheck() -> impl IntoResponse {
    Json(response_message(&Status::Success, "Hello world!!"))
}
