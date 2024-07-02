use std::sync::Arc;

use axum::{
    extract::State,
    http::{header, HeaderMap, Response, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};

use crate::{
    api::utils::status::{response_message, Status},
    application::AppState,
    domain::{auth_service::AuthService, repositories::auth_repository::AuthRepository},
    helper::redis_helper,
    model::{api_error::ApiError, api_response::ApiResponse, auth_middleware::AuthMiddleware},
};

pub async fn logout_handler<AR: AuthRepository, AS: AuthService>(
    Extension(auth_guard): Extension<AuthMiddleware>,
    State(data): State<Arc<AppState<AR, AS>>>,
) -> Result<impl IntoResponse, ApiError> {
    redis_helper::delete_token_data(&data.redis, &auth_guard.access_token_uuid.to_string())
        .await
        .map_err(|_| {
            let error_message = response_message(&Status::Failure, "Redis failed to delete token");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_message))
        })
        .unwrap();

    let headers = set_cookies_in_header().unwrap();

    let mut response = Response::new(
        ApiResponse::<String>::success_message("User logged out".to_string())
            .to_json()
            .to_string(),
    );
    response.headers_mut().extend(headers);
    Ok(response)
}

fn set_cookies_in_header() -> Result<HeaderMap, (StatusCode, Json<serde_json::Value>)> {
    let access_cookie = Cookie::build(("access_token", ""))
        .path("/")
        .max_age(time::Duration::minutes(-1))
        .same_site(SameSite::Lax)
        .http_only(true)
        .to_string()
        .parse()
        .map_err(|_| {
            let error_message = response_message(&Status::Failure, "Failed to parse access cookie");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_message))
        })?;

    let logged_in_cookie = Cookie::build(("logged_in", "true"))
        .path("/")
        .max_age(time::Duration::minutes(-1))
        .same_site(SameSite::Lax)
        .http_only(false)
        .to_string()
        .parse()
        .map_err(|_| {
            let error_message =
                response_message(&Status::Failure, "Failed to parse logged in cookie");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_message))
        })?;

    let mut headers = HeaderMap::new();
    headers.append(header::SET_COOKIE, access_cookie);
    headers.append(header::SET_COOKIE, logged_in_cookie);
    Ok(headers)
}
