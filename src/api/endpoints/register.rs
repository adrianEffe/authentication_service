use crate::api::schemas::register_user::RegisterUserSchema;
use crate::application::{ApiResponse, AppState};
use crate::domain::repositories::auth_repository::AuthRepository;
use crate::model::api_error::ApiError;
use crate::model::user::FilteredUser;
use axum::{extract::State, Json};
use std::sync::Arc;

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
