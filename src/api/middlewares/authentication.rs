use crate::{
    api::utils::{
        jwt::verify_jwt,
        status::{response_message, Status},
    },
    application::AppState,
    domain::repositories::auth_repository::AuthRepository,
    model::{auth_middleware::AuthMiddleware, token::TokenDetails, user::User},
};
use axum::{
    body::Body,
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::CookieJar;
use redis::{AsyncCommands, Client};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

pub async fn auth<AR: AuthRepository>(
    cookie_jar: CookieJar,
    State(state): State<Arc<AppState<AR>>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let access_token = extract_access_token(cookie_jar, &req)?;

    let access_token_details = match verify_jwt(&state.env.access_token_public_key, &access_token) {
        Ok(token_details) => token_details,
        Err(e) => {
            let error_response = response_message(&Status::Failure, &e.to_string());
            return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
        }
    };

    verify_active_session(&state.redis, &access_token_details).await?;

    let user = fetch_user_from_db(&state.db, access_token_details.user_id).await?;

    req.extensions_mut().insert(AuthMiddleware {
        user,
        access_token_uuid: access_token_details.token_uuid,
    });
    Ok(next.run(req).await)
}

async fn verify_active_session(
    redis: &Client,
    access_token_details: &TokenDetails,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    let access_token_uuid = uuid::Uuid::parse_str(&access_token_details.token_uuid.to_string())
        .map_err(|_| {
            let error_response = response_message(&Status::Failure, "Invalid token");
            (StatusCode::UNAUTHORIZED, Json(error_response))
        })?;

    let mut redis_client = redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| {
            let message = format!("Redis error: {}", e);
            let error_response = response_message(&Status::Failure, &message);
            (StatusCode::UNAUTHORIZED, Json(error_response))
        })?;

    redis_client
        .get::<_, String>(access_token_uuid.to_string())
        .await
        .map_err(|_| {
            let error_response =
                response_message(&Status::Failure, "Token is invalid or session has expired");
            (StatusCode::UNAUTHORIZED, Json(error_response))
        })?;
    Ok(())
}

async fn fetch_user_from_db(
    db: &Pool<Postgres>,
    user_id: uuid::Uuid,
) -> Result<User, (StatusCode, Json<serde_json::Value>)> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
        .fetch_optional(db)
        .await
        .map_err(|e| {
            let message = format!("Error fetching from database: {}", e);
            let error_response = response_message(&Status::Failure, &message);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    user.ok_or_else(|| {
        let error_response = response_message(
            &Status::Failure,
            "The user belonging to this token no longer exists",
        );
        (StatusCode::UNAUTHORIZED, Json(error_response))
    })
}

fn extract_access_token(
    cookie_jar: CookieJar,
    req: &Request<Body>,
) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
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

    access_token.ok_or_else(|| {
        let error_response = response_message(&Status::Failure, "You are not logged in");
        (StatusCode::UNAUTHORIZED, Json(error_response))
    })
}
