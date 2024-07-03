use crate::{
    application::AppState,
    domain::{
        auth_service::AuthService,
        model::auth::{AuthRequest, AuthorizationError},
    },
    model::api_error::ApiError,
};

use axum::{
    body::Body,
    extract::State,
    http::{header, Request},
    middleware::Next,
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use std::sync::Arc;

pub async fn auth<AS: AuthService>(
    cookie_jar: CookieJar,
    State(state): State<Arc<AppState<AS>>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, ApiError> {
    let access_token = extract_access_token(cookie_jar, &req).map_err(ApiError::from)?;
    let domain_request = AuthRequest::new(access_token);

    let auth_middleware = state.auth_service.auth(&domain_request).await?;

    req.extensions_mut().insert(auth_middleware);
    Ok(next.run(req).await)
}

fn extract_access_token(
    cookie_jar: CookieJar,
    req: &Request<Body>,
) -> Result<String, AuthorizationError> {
    let access_token = cookie_jar
        .get("access_token")
        .map(|cookie| cookie.value().to_string())
        .or_else(|| {
            req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .and_then(|auth_value| {
                    auth_value
                        .strip_prefix("Bearer ")
                        .map(|token| token.to_owned())
                })
        });

    access_token.ok_or_else(|| AuthorizationError::InvalidCredentials {
        reason: "You are not logged in".to_string(),
    })
}
