use crate::{
    api::utils::jwt::verify_jwt,
    application::AppState,
    domain::{auth_service::AuthService, repositories::auth_repository::AuthRepository},
    model::{
        api_error::ApiError,
        auth::{AuthRequest, AuthorizationError},
        auth_middleware::AuthMiddleware,
        token::TokenDetails,
    },
};
use anyhow::anyhow;
use axum::{
    body::Body,
    extract::State,
    http::{header, Request},
    middleware::Next,
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use redis::{AsyncCommands, Client};
use std::sync::Arc;

pub async fn auth<AR: AuthRepository, AS: AuthService>(
    cookie_jar: CookieJar,
    State(state): State<Arc<AppState<AR, AS>>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, ApiError> {
    let access_token = extract_access_token(cookie_jar, &req).map_err(ApiError::from)?;

    let access_token_details = verify_jwt(&state.env.access_token_public_key, &access_token)
        .map_err(|_| {
            ApiError::from(AuthorizationError::InvalidCredentials {
                reason: "Failed to verify token integrity".to_string(),
            })
        })?;

    verify_active_session(&state.redis, &access_token_details).await?;

    let request = AuthRequest::new(access_token_details.user_id);
    let user = state.auth_repository.auth(&request).await?;

    req.extensions_mut().insert(AuthMiddleware {
        user,
        access_token_uuid: access_token_details.token_uuid,
    });
    Ok(next.run(req).await)
}

async fn verify_active_session(
    redis: &Client,
    access_token_details: &TokenDetails,
) -> Result<(), AuthorizationError> {
    let access_token_uuid = uuid::Uuid::parse_str(&access_token_details.token_uuid.to_string())
        .map_err(|_| AuthorizationError::InvalidCredentials {
            reason: "Invalid token".to_string(),
        })?;

    let mut redis_client = redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| anyhow!(e).context("Redis error"))?;

    redis_client
        .get::<_, String>(access_token_uuid.to_string())
        .await
        .map_err(|_| AuthorizationError::InvalidCredentials {
            reason: "Token is invalid or session has expired".to_string(),
        })?;
    Ok(())
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
