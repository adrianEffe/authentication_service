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
    model::user::User,
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
    // TODO: extract validate password
    let is_valid = is_valid(&body.password, &user.password);
    if !is_valid {
        let error_response = response_message(&Status::Failure, "Invalid email or password");
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    let access_token_details = generate_jwt(
        user.id,
        data.env.access_token_max_age,
        &data.env.access_token_private_key,
    )
    .map_err(|_| {
        let error_message = response_message(&Status::Failure, "Failed to generate token");
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_message))
    })?;

    //TODO: extract cookie builder function

    // Remove unwraps
    let access_cookie = Cookie::build((
        "access_token",
        access_token_details.token.clone().unwrap_or_default(),
    ))
    .path("/")
    .max_age(time::Duration::minutes(data.env.access_token_max_age))
    .same_site(SameSite::Lax)
    .http_only(true);

    let logged_in_cookie = Cookie::build(("logged_in", "true"))
        .path("/")
        .max_age(time::Duration::minutes(data.env.access_token_max_age))
        .same_site(SameSite::Lax)
        .http_only(false);

    // remove unwraps
    let mut response = Response::new(
        response_data(
            &Status::Success,
            "access_token",
            access_token_details.token.unwrap(),
        )
        .to_string(),
    );

    let mut headers = HeaderMap::new();
    headers.append(
        header::SET_COOKIE,
        access_cookie.to_string().parse().unwrap(),
    );
    headers.append(
        header::SET_COOKIE,
        logged_in_cookie.to_string().parse().unwrap(),
    );

    response.headers_mut().extend(headers);
    Ok(response)
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
