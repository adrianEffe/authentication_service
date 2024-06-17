use crate::api::schemas::register_user::RegisterUserSchema;
use crate::api::utils::status::{response_data, response_message, Status};
use crate::application::AppState;
use crate::model::user::User;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use std::sync::Arc;

pub async fn register_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<RegisterUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (email, password) VALUES ($1, $2) RETURNING *",
        body.email.to_string().to_ascii_lowercase(),
        body.password.to_string()
    )
    .fetch_one(&data.db)
    .await
    .map_err(|e| {
        let message = format!("Database error: {}", e);
        let error_response = response_message(&Status::Failure, &message);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let user_response = response_data(&Status::Success, "user", user);

    Ok(Json(user_response))
}
