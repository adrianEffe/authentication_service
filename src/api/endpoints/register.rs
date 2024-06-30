use crate::api::{
    schemas::register_user::RegisterUserSchema,
    utils::{
        password_hasher,
        status::{response_data, response_message, Status},
    },
};
use crate::application::{ApiError, ApiResponse, AppState};
use crate::model::register_user::{RegisterUserError, RegisterUserRequest};
use crate::model::user::{FilteredUser, User};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::future::Future;
use std::sync::Arc;

pub trait AuthRepository: Send + Sync + 'static {
    fn register(
        &self,
        request: &RegisterUserRequest,
    ) -> impl Future<Output = Result<FilteredUser, RegisterUserError>> + Send;
}

pub async fn register_handler<AR: AuthRepository>(
    State(state): State<Arc<AppState<AR>>>,
    Json(body): Json<RegisterUserSchema>,
) -> Result<ApiResponse<FilteredUser>, ApiError> {
    let domain_request = body.try_into_domain()?;

    state
        .auth_repository
        .register(&domain_request)
        .await
        .map_err(ApiError::from)
        .map(ApiResponse::success)
}
