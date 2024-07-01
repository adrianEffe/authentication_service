use axum::Extension;

use crate::model::{
    api_error::ApiError, api_response::ApiResponse, auth_middleware::AuthMiddleware,
    user::FilteredUser,
};

pub async fn get_me_handler(
    Extension(jwt): Extension<AuthMiddleware>,
) -> Result<ApiResponse<FilteredUser>, ApiError> {
    let filtered_user = FilteredUser::from(&jwt.user);
    Ok(ApiResponse::success(filtered_user))
}
