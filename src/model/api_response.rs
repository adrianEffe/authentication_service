use std::fmt::Debug;

use crate::api::utils::status::Status;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    status: Status,
    data: Option<T>,
    message: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        ApiResponse {
            status: Status::Success,
            data: Some(data),
            message: None,
        }
    }

    pub fn to_json(self) -> serde_json::Value {
        serde_json::json!(self)
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status_code = match self.status {
            Status::Success => StatusCode::OK,
            Status::Failure => StatusCode::BAD_REQUEST,
        };
        (status_code, Json(self)).into_response()
    }
}
