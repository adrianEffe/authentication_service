use crate::api::{
    schemas::register_user::RegisterUserSchema,
    utils::password_hasher,
    utils::status::{response_data, response_message, Status},
};
use crate::application::AppState;
use crate::model::user::User;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

pub async fn register_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<RegisterUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    check_user_exists(&data.db, &body.email).await?;

    let hashed_password = password_hasher::hash_password(&body.password).map_err(|e| {
        let message = format!("Error hashing password: {}", e);
        let error_response = response_message(&Status::Failure, &message);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let user =
        insert_user_in_db(&data.db, &body.email.to_ascii_lowercase(), &hashed_password).await?;

    // TODO: return a lean version of user w/o password
    let user_response = response_data(&Status::Success, "user", user);

    Ok(Json(user_response))
}

async fn insert_user_in_db(
    db: &Pool<Postgres>,
    email: &str,
    hashed_password: &str,
) -> Result<User, (StatusCode, Json<serde_json::Value>)> {
    sqlx::query_as!(
        User,
        "INSERT INTO users (email, password) VALUES ($1, $2) RETURNING *",
        email.to_string().to_ascii_lowercase(),
        hashed_password
    )
    .fetch_one(db)
    .await
    .map_err(|e| {
        let message = format!("Database error: {}", e);
        let error_response = response_message(&Status::Failure, &message);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })
}

async fn check_user_exists(
    db: &Pool<Postgres>,
    email: &str,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    let user_exists: Option<bool> =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
            .bind(email.to_owned().to_ascii_lowercase())
            .fetch_one(db)
            .await
            .map_err(|e| {
                let message = format!("Database error: {}", e);
                let error_response = response_message(&Status::Failure, &message);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })?;
    if let Some(exists) = user_exists {
        if exists {
            let message = "User with this email already exists";
            let error_response = response_message(&Status::Failure, message);
            return Err((StatusCode::CONFLICT, Json(error_response)));
        } else {
            return Ok(());
        }
    }
    Ok(())
}
