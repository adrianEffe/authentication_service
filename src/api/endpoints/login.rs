// use crate::{api::schemas::register_user::RegisterUserSchema, application::AppState};
// use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
// use std::sync::Arc;
//
// pub async fn login_handler(
//     State(data): State<Arc<AppState>>,
//     Json(body): Json<RegisterUserSchema>,
// ) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
// }
