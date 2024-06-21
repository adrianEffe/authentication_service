use crate::{
    api::{
        schemas::login_user::LoginUserSchema,
        utils::{
            jwt::generate_jwt,
            password_hasher::is_valid,
            status::{response_data, response_message, Status},
        },
    },
    application::AppState,
    helper::redis_helper,
    model::{login_response::LoginResponse, token::TokenDetails, user::User},
};
use axum::{
    extract::State,
    http::{header, HeaderMap, Response, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

pub async fn login_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<LoginUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user = fetch_user(&data.db, &body.email).await?;

    let is_valid = is_valid(&body.password, &user.password);
    if !is_valid {
        let error_response = response_message(&Status::Failure, "Invalid email or password");
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    let access_token_details = generate_access_token(
        user.id,
        data.env.access_token_max_age,
        &data.env.access_token_private_key,
    )?;

    redis_helper::save_token_data(&data, &access_token_details, data.env.access_token_max_age)
        .await
        .map_err(|e| {
            let message = format!("Redis error: {}", e);
            let error_response = response_message(&Status::Failure, &message);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    let access_token = access_token_details.token.ok_or_else(|| {
        let error_response = response_message(&Status::Failure, "Failed to generate token");
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let mut response = Response::new(
        response_data(
            &Status::Success,
            LoginResponse {
                access_token: access_token.to_owned(),
            },
        )
        .to_string(),
    );

    let headers = set_cookies_in_header(&access_token, data.env.access_token_max_age)?;

    response.headers_mut().extend(headers);
    Ok(response)
}

fn generate_access_token(
    user_id: uuid::Uuid,
    max_age: i64,
    private_key: &str,
) -> Result<TokenDetails, (StatusCode, Json<serde_json::Value>)> {
    generate_jwt(user_id, max_age, private_key).map_err(|_| {
        let error_message = response_message(&Status::Failure, "Failed to generate jwt detail");
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_message))
    })
}

fn set_cookies_in_header(
    access_token: &str,
    max_age: i64,
) -> Result<HeaderMap, (StatusCode, Json<serde_json::Value>)> {
    let access_cookie = Cookie::build(("access_token", access_token))
        .path("/")
        .max_age(time::Duration::minutes(max_age))
        .same_site(SameSite::Lax)
        .http_only(true)
        .to_string()
        .parse()
        .map_err(|_| {
            let error_message = response_message(&Status::Failure, "Failed to parse access cookie");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_message))
        })?;

    let logged_in_cookie = Cookie::build(("logged_in", "true"))
        .path("/")
        .max_age(time::Duration::minutes(max_age))
        .same_site(SameSite::Lax)
        .http_only(false)
        .to_string()
        .parse()
        .map_err(|_| {
            let error_message =
                response_message(&Status::Failure, "Failed to parse logged in cookie");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_message))
        })?;

    let mut headers = HeaderMap::new();
    headers.append(header::SET_COOKIE, access_cookie);
    headers.append(header::SET_COOKIE, logged_in_cookie);
    Ok(headers)
}

async fn fetch_user(
    db: &Pool<Postgres>,
    email: &str,
) -> Result<User, (StatusCode, Json<serde_json::Value>)> {
    sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE email = $1",
        email.to_ascii_lowercase()
    )
    .fetch_optional(db)
    .await
    .map_err(|e| {
        let message = format!("Database error: {}", e);
        let error_message = response_message(&Status::Failure, &message);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_message))
    })?
    .ok_or_else(|| {
        let error_message = response_message(&Status::Failure, "Invalid email or password");
        (StatusCode::BAD_REQUEST, Json(error_message))
    })
}
