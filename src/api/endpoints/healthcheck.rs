use axum::response::IntoResponse;
use axum::Json;

pub async fn healthcheck() -> impl IntoResponse {
    Json("Hello World")
}
