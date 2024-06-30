use crate::api::utils::status::Status;
use crate::model::api_error::ApiError;
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

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        ApiResponse {
            status: Status::Success,
            data: Some(data),
            message: None,
        }
    }

    pub fn error(error: ApiError) -> Self {
        ApiResponse {
            status: Status::Failure,
            data: None,
            message: Some(error.to_string()),
        }
    }
}

// TODO: - make this better so you can pass proper status codes
impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status_code = match self.status {
            Status::Success => StatusCode::OK,
            Status::Failure => StatusCode::BAD_REQUEST,
        };
        (status_code, Json(self)).into_response()
    }
}
