use super::utils::status::{response_message, Status};
use axum::response::IntoResponse;
use axum::Json;

pub async fn healthcheck() -> impl IntoResponse {
    Json(response_message(&Status::Success, "Hello world!!"))
}
