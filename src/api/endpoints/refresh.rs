use std::sync::Arc;

use anyhow::{anyhow, Result};
use axum::{
    extract::State,
    http::{header, HeaderMap, Response},
    response::IntoResponse,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};

use crate::{
    api::model::{api_error::ApiError, api_response::ApiResponse},
    application::AppState,
    domain::{
        auth_service::AuthService,
        model::refresh_token::{RefreshRequest, RefreshResponse, RefreshTokenError},
    },
};

pub async fn refresh_access_token_handler<AS: AuthService>(
    cookie_jar: CookieJar,
    State(state): State<Arc<AppState<AS>>>,
) -> Result<impl IntoResponse, ApiError> {
    let refresh_token =
        extract_refresh_token(cookie_jar).map_err(|_| RefreshTokenError::MissingCredentials)?;

    let domain_request = RefreshRequest::new(refresh_token);

    let refresh_response = state.auth_service.refresh(&domain_request).await?;

    let headers = set_cookies_in_header(&refresh_response).map_err(|e| {
        ApiError::from(RefreshTokenError::Unknown(
            anyhow!(e).context("Failed to set cookies in header"),
        ))
    })?;

    let mut response = Response::new(ApiResponse::success(refresh_response).to_json().to_string());
    response.headers_mut().extend(headers);

    Ok(response)
}

fn extract_refresh_token(cookie_jar: CookieJar) -> Result<String> {
    cookie_jar
        .get("refresh_token")
        .map(|cookie| cookie.value().to_string())
        .ok_or(anyhow!("Failed to extract refresh token from cookie"))
}

fn set_cookies_in_header(details: &RefreshResponse) -> anyhow::Result<HeaderMap> {
    let access_cookie = Cookie::build(("access_token", details.access_token.to_string()))
        .path("/")
        .max_age(time::Duration::minutes(details.access_token_max_age))
        .same_site(SameSite::Lax)
        .http_only(true)
        .to_string()
        .parse()?;

    let logged_in_cookie = Cookie::build(("logged_in", "true"))
        .path("/")
        .max_age(time::Duration::minutes(details.access_token_max_age))
        .same_site(SameSite::Lax)
        .http_only(false)
        .to_string()
        .parse()?;

    let mut headers = HeaderMap::new();
    headers.append(header::SET_COOKIE, access_cookie);
    headers.append(header::SET_COOKIE, logged_in_cookie);
    Ok(headers)
}
