use crate::{
    api::model::{api_error::ApiError, api_response::ApiResponse},
    api::schemas::register_user::RegisterUserSchema,
    application::AppState,
    domain::{auth_service::AuthService, model::user::FilteredUser},
};
use axum::{extract::State, Json};
use std::sync::Arc;

pub async fn register_handler<AS: AuthService>(
    State(state): State<Arc<AppState<AS>>>,
    Json(body): Json<RegisterUserSchema>,
) -> Result<ApiResponse<FilteredUser>, ApiError> {
    let domain_request = body.try_into_domain()?;

    state
        .auth_service
        .register(&domain_request)
        .await
        .map_err(ApiError::from)
        .map(ApiResponse::success)
}
