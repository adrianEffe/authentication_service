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
    helper::redis_helper,
    model::auth_middleware::AuthMiddleware,
};

pub async fn logout_handler(
    Extension(auth_guard): Extension<AuthMiddleware>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    redis_helper::delete_token_data(&data.redis, &auth_guard.access_token_uuid.to_string())
        .await
        .map_err(|_| {
            let error_message = response_message(&Status::Failure, "Redis failed to delete token");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_message))
        })?;

    let headers = set_cookies_in_header()?;

    let mut response =
        Response::new(response_message(&Status::Success, "User logged out").to_string());
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
