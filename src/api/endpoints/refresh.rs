use std::sync::Arc;

use anyhow::{anyhow, Result};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;

use crate::{
    api::model::api_error::ApiError, application::AppState, domain::auth_service::AuthService,
};

pub async fn refresh_access_token_handler<AS: AuthService>(
    cookie_jar: CookieJar,
    State(state): State<Arc<AppState<AS>>>,
) -> Result<impl IntoResponse, ApiError> {
    let refresh_token = extract_refresh_token(cookie_jar)
        // .map_err(RefreshTokenError::MissingCredentials)
        .unwrap(); //TODO: //handle error

    Ok(Response::new("todo".to_string()))
}

fn extract_refresh_token(cookie_jar: CookieJar) -> Result<String> {
    cookie_jar
        .get("refresh_token")
        .map(|cookie| cookie.value().to_string())
        .ok_or(anyhow!("Failed to extract refresh token from cookie"))
}
