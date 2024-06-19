use crate::{
    api::utils::{
        jwt::verify_jwt,
        status::{response_message, Status},
    },
    application::AppState,
    model::auth_middleware::AuthMiddleware,
    model::user::User,
};
use axum::{
    body::Body,
    extract::State,
    http::{header, Request},
    middleware::Next,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::CookieJar;
use reqwest::StatusCode;
use std::sync::Arc;

pub async fn auth(
    cookie_jar: CookieJar,
    State(data): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
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

    let access_token = access_token.ok_or_else(|| {
        let error_response = response_message(&Status::Failure, "You are not logged in");
        (StatusCode::UNAUTHORIZED, Json(error_response))
    })?;

    let access_token_details = match verify_jwt(&data.env.access_token_public_key, &access_token) {
        Ok(token_details) => token_details,
        Err(e) => {
            let error_response = response_message(&Status::Failure, &e.to_string());
            return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
        }
    };

    // TODO: Could have a redis instance to check for revoked tokens
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id = $1",
        access_token_details.user_id
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|e| {
        let message = format!("Error fetching from database: {}", e);
        let error_response = response_message(&Status::Failure, &message);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let user = user.ok_or_else(|| {
        let error_response = response_message(
            &Status::Failure,
            "The user belonging to this token no longer exists",
        );
        (StatusCode::UNAUTHORIZED, Json(error_response))
    })?;

    req.extensions_mut().insert(AuthMiddleware {
        user,
        access_token_uuid: access_token_details.token_uuid,
    });
    Ok(next.run(req).await)
}
