use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/api/healthcheck", get(healthcheck));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn healthcheck() -> String {
    "Hello World".to_string()
}
