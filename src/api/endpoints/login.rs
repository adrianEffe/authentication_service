use crate::{
    api::{
        schemas::login_user::LoginUserSchema,
        utils::{jwt::generate_jwt, password_hasher::is_valid},
    },
    application::AppState,
    domain::repositories::auth_repository::AuthRepository,
    helper::redis_helper,
    model::{
        api_error::ApiError, api_response::ApiResponse, login_response::LoginResponse,
        login_user::LoginUserError,
    },
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

pub async fn login_handler<AR: AuthRepository>(
    State(state): State<Arc<AppState<AR>>>,
    Json(body): Json<LoginUserSchema>,
) -> Result<impl IntoResponse, ApiError> {
    let domain_request = body.try_into_domain()?;
    let user = state.auth_repository.login(&domain_request).await?;

    let is_valid = is_valid(&body.password, &user.password);
    if !is_valid {
        return Err(ApiError::from(LoginUserError::InvalidCredentials));
    }

    let access_token_details = generate_jwt(
        user.id,
        state.env.access_token_max_age,
        &state.env.access_token_private_key,
    )
    .map_err(|e| {
        ApiError::from(LoginUserError::Unknown(
            anyhow!(e).context("Failed to generate jwt token"),
        ))
    })?;

    // TODO: - abstract redis away
    redis_helper::save_token_data(
        &state,
        &access_token_details,
        state.env.access_token_max_age,
    )
    .await
    .map_err(|e| {
        ApiError::from(LoginUserError::Unknown(
            anyhow!(e).context("Failed to save token to redis"),
        ))
    })?;

    let access_token = access_token_details.token.ok_or_else(|| {
        ApiError::from(LoginUserError::Unknown(anyhow!("Failed to generate token")))
    })?;

    let mut response = Response::new(
        ApiResponse::success(LoginResponse {
            access_token: access_token.to_owned(),
        })
        .to_json()
        .to_string(),
    );

    let headers =
        set_cookies_in_header(&access_token, state.env.access_token_max_age).map_err(|e| {
            ApiError::from(LoginUserError::Unknown(
                anyhow!(e).context("Failed to set cookies in header"),
            ))
        })?;

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
