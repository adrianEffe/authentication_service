use axum::{response::IntoResponse, routing::get, Json, Router};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/api/healthcheck", get(healthcheck));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn healthcheck() -> impl IntoResponse {
    let json_response = serde_json::json!({
        "status": "success",
        "message": "Hello World"
    });
    Json(json_response)
}
