use crate::{
    api::utils::status::{response_message, Status},
    application::AppState,
    model::user::User,
};
use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, Request},
    middleware::Next,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::CookieJar;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthMiddleware {
    pub user: User,
    pub access_token_uuid: uuid::Uuid,
}

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
                    if auth_value.starts_with("Bearer ") {
                        Some(auth_value[7..].to_owned())
                    } else {
                        None
                    }
                })
        });

    let access_token = access_token.ok_or_else(|| {
        let error_response = response_message(&Status::Failure, "You are not logged in");
        (StatusCode::UNAUTHORIZED, Json(error_response))
    })?;
    Ok(next.run(req).await)
}
