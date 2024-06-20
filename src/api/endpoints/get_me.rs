use axum::{http::StatusCode, response::IntoResponse, Extension, Json};

use crate::{
    api::utils::status::{response_data, Status},
    model::auth_middleware::AuthMiddleware,
};

pub async fn get_me_handler(
    Extension(jwt): Extension<AuthMiddleware>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(response_data(&Status::Success, "user", jwt.user)))
}
