use crate::{
    api::schemas::login_user::LoginUserSchema,
    application::AppState,
    domain::{auth_service::AuthService, repositories::auth_repository::AuthRepository},
    model::{api_error::ApiError, api_response::ApiResponse, login_user::LoginUserError},
};
use anyhow::anyhow;
use axum::{
    extract::State,
    http::{header, HeaderMap, Response},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use std::sync::Arc;

pub async fn login_handler<AR: AuthRepository, AS: AuthService>(
    State(state): State<Arc<AppState<AR, AS>>>,
    Json(body): Json<LoginUserSchema>,
) -> Result<impl IntoResponse, ApiError> {
    let domain_request = body.try_into_domain()?;
    let login_response = state
        .auth_service
        .login(&domain_request)
        .await
        .map_err(ApiError::from)?;

    let headers = set_cookies_in_header(
        &login_response.access_token,
        login_response.access_token_max_age,
    )
    .map_err(|e| {
        ApiError::from(LoginUserError::Unknown(
            anyhow!(e).context("Failed to set cookies in header"),
        ))
    })?;

    let mut response = Response::new(ApiResponse::success(login_response).to_json().to_string());
    response.headers_mut().extend(headers);

    Ok(response)
}

fn set_cookies_in_header(access_token: &str, max_age: i64) -> anyhow::Result<HeaderMap> {
    let access_cookie = Cookie::build(("access_token", access_token))
        .path("/")
        .max_age(time::Duration::minutes(max_age))
        .same_site(SameSite::Lax)
        .http_only(true)
        .to_string()
        .parse()?;

    let logged_in_cookie = Cookie::build(("logged_in", "true"))
        .path("/")
        .max_age(time::Duration::minutes(max_age))
        .same_site(SameSite::Lax)
        .http_only(false)
        .to_string()
        .parse()?;

    let mut headers = HeaderMap::new();
    headers.append(header::SET_COOKIE, access_cookie);
    headers.append(header::SET_COOKIE, logged_in_cookie);
    Ok(headers)
}
