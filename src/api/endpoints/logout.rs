use std::sync::Arc;

use anyhow::{anyhow, Result};
use axum::{
    extract::State,
    http::{header, HeaderMap, Response},
    response::IntoResponse,
    Extension,
};
use axum_extra::extract::cookie::{Cookie, SameSite};

use crate::{
    application::AppState,
    domain::auth_service::AuthService,
    model::{
        api_error::ApiError,
        api_response::ApiResponse,
        auth::AuthorizationError,
        auth_middleware::AuthMiddleware,
        logout::{LogoutRequest, LogoutResponse},
    },
};

pub async fn logout_handler<AS: AuthService>(
    Extension(auth_guard): Extension<AuthMiddleware>,
    State(state): State<Arc<AppState<AS>>>,
) -> Result<impl IntoResponse, ApiError> {
    let domain_request = LogoutRequest::new(auth_guard.access_token_uuid);

    let response = state
        .auth_service
        .logout(&domain_request)
        .await
        .map_err(ApiError::from)?;

    let mut response = Response::new(
        ApiResponse::<LogoutResponse>::success_message(response)
            .to_json()
            .to_string(),
    );

    let headers = set_cookies_in_header().map_err(|e| {
        AuthorizationError::Unknown(anyhow!(e).context("Failed to set cookies in header"))
    })?;

    response.headers_mut().extend(headers);

    Ok(response)
}

fn set_cookies_in_header() -> Result<HeaderMap> {
    let access_cookie = Cookie::build(("access_token", ""))
        .path("/")
        .max_age(time::Duration::minutes(-1))
        .same_site(SameSite::Lax)
        .http_only(true)
        .to_string()
        .parse()?;

    let logged_in_cookie = Cookie::build(("logged_in", "true"))
        .path("/")
        .max_age(time::Duration::minutes(-1))
        .same_site(SameSite::Lax)
        .http_only(false)
        .to_string()
        .parse()?;

    let mut headers = HeaderMap::new();
    headers.append(header::SET_COOKIE, access_cookie);
    headers.append(header::SET_COOKIE, logged_in_cookie);
    Ok(headers)
}
