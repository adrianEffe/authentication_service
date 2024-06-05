use axum::{response::IntoResponse, routing::get, Json, Router};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/api/healthcheck", get(healthcheck));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn healthcheck() -> impl IntoResponse {
    response_message(&Status::Success, "Hello world!!");
    Json(())
}
enum Status {
    Success,
    Failure,
}

impl Status {
    fn raw_value(&self) -> String {
        match &self {
            Status::Success => String::from("success"),
            Status::Failure => String::from("failure"),
        }
    }
}

fn response_message(status: &Status, message: &str) {
    serde_json::json!({
        "status": status.raw_value(),
        "message": message,
    });
}
