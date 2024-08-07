use authentication_service::{
    api::utils::status::Status, application::run, domain::model::user::FilteredUser,
    helper::config::Config,
};
use dotenv::dotenv;
use redis::{AsyncCommands, Client};
use reqwest::{header::AUTHORIZATION, StatusCode};
use serde::Deserialize;
use sqlx::{postgres::PgPoolOptions, Executor, Pool, Postgres};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::test]
async fn test_register_success() {
    let address = spawn_server().await;

    let url = format!("http://{}/api/register", address);
    let client = reqwest::Client::new();

    let email = "register_success@test.com";
    let body = serde_json::json!({
        "email": email,
        "password": "12345678"
    });

    let response: GenericResponse<FilteredUser> = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .expect("failed at send")
        .json()
        .await
        .expect("failed at json");

    clean_up_db(|db| async move {
        db.execute(sqlx::query!("DELETE FROM users WHERE email = $1", email))
            .await
            .unwrap();
    })
    .await;

    assert_eq!(response.data.unwrap().email, email);
}

#[tokio::test]
async fn test_register_existing_user_failure() {
    let address = spawn_server().await;

    let url = format!("http://{}/api/register", address);
    let client = reqwest::Client::new();

    let email = "existing_failure@test.com";
    let body = serde_json::json!({
        "email": email,
        "password": "12345678"
    });

    let _ = client.post(&url).json(&body).send().await;

    let response = client.post(&url).json(&body).send().await.unwrap();

    clean_up_db(|db| async move {
        db.execute(sqlx::query!("DELETE FROM users WHERE email = $1", email))
            .await
            .unwrap();
    })
    .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_login_success() {
    let address = spawn_server().await;

    let register_url = format!("http://{}/api/register", address);
    let login_url = format!("http://{}/api/login", address);
    let client = reqwest::Client::new();

    let email = "login_success@test.com";
    let body = serde_json::json!({
        "email": email,
        "password": "12345678"
    });

    let _ = client.post(&register_url).json(&body).send().await;

    let response: GenericResponse<AccessTokenData> = client
        .post(&login_url)
        .json(&body)
        .send()
        .await
        .expect("failed at send")
        .json()
        .await
        .expect("failed at json");

    clean_up_db(|db| async move {
        db.execute(sqlx::query!("DELETE FROM users WHERE email = $1", email))
            .await
            .unwrap();
    })
    .await;

    assert!(!response.data.unwrap().access_token.is_empty());
}

#[tokio::test]
async fn test_login_failure() {
    let address = spawn_server().await;

    let login_url = format!("http://{}/api/login", address);
    let client = reqwest::Client::new();

    let email = "login_failure@test.com";
    let body = serde_json::json!({
        "email": email,
        "password": "12345678"
    });

    let response = client.post(&login_url).json(&body).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_revoked_token() {
    let address = spawn_server().await;

    let register_url = format!("http://{}/api/register", address);
    let login_url = format!("http://{}/api/login", address);
    let get_me_url = format!("http://{}/api/users/me", address);
    let client = reqwest::Client::new();

    let email = "login_revoked_failure@test.com";
    let body = serde_json::json!({
        "email": email,
        "password": "12345678"
    });

    let _ = client.post(&register_url).json(&body).send().await;

    let response: GenericResponse<AccessTokenData> = client
        .post(&login_url)
        .json(&body)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let token = response.data.unwrap().access_token;

    revoke_token_from_redis(&token).await;

    let response = client
        .get(&get_me_url)
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    clean_up_db(|db| async move {
        db.execute(sqlx::query!("DELETE FROM users WHERE email = $1", email))
            .await
            .unwrap();
    })
    .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_refresh_token_success() {
    let address = spawn_server().await;

    let register_url = format!("http://{}/api/register", address);
    let login_url = format!("http://{}/api/login", address);
    let refresh_token_url = format!("http://{}/api/refresh", address);

    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();

    let email = "refresh_token_success@test.com";
    let body = serde_json::json!({
        "email": email,
        "password": "12345678"
    });

    let _ = client.post(&register_url).json(&body).send().await;

    let _ = client.post(&login_url).json(&body).send().await;

    let response: GenericResponse<AccessTokenData> = client
        .get(&refresh_token_url)
        .json(&body)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    clean_up_db(|db| async move {
        db.execute(sqlx::query!("DELETE FROM users WHERE email = $1", email))
            .await
            .unwrap();
    })
    .await;

    assert!(!response.data.unwrap().access_token.is_empty());
}

#[tokio::test]
async fn test_refresh_token_failure() {
    let address = spawn_server().await;

    let register_url = format!("http://{}/api/register", address);
    let login_url = format!("http://{}/api/login", address);
    let refresh_token_url = format!("http://{}/api/refresh", address);

    let client = reqwest::Client::new();

    let email = "refresh_token_failure@test.com";
    let body = serde_json::json!({
        "email": email,
        "password": "12345678"
    });

    let _ = client.post(&register_url).json(&body).send().await;

    let _ = client.post(&login_url).json(&body).send().await;

    let response = client
        .get(&refresh_token_url)
        .json(&body)
        .send()
        .await
        .unwrap();

    clean_up_db(|db| async move {
        db.execute(sqlx::query!("DELETE FROM users WHERE email = $1", email))
            .await
            .unwrap();
    })
    .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED)
}

#[tokio::test]
async fn test_get_me_success() {
    let address = spawn_server().await;

    let register_url = format!("http://{}/api/register", address);
    let login_url = format!("http://{}/api/login", address);
    let get_me_url = format!("http://{}/api/users/me", address);
    let client = reqwest::Client::new();

    let email = "get_me_success@test.com";
    let body = serde_json::json!({
        "email": email,
        "password": "12345678"
    });

    let _ = client.post(&register_url).json(&body).send().await;

    let response: GenericResponse<AccessTokenData> = client
        .post(&login_url)
        .json(&body)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let token = response.data.unwrap().access_token;

    let response: GenericResponse<FilteredUser> = client
        .get(&get_me_url)
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    clean_up_db(|db| async move {
        db.execute(sqlx::query!("DELETE FROM users WHERE email = $1", email))
            .await
            .unwrap();
    })
    .await;

    assert_eq!(response.data.unwrap().email, email);
}

#[tokio::test]
async fn test_get_me_failure() {
    let address = spawn_server().await;

    let get_me_url = format!("http://{}/api/users/me", address);
    let client = reqwest::Client::new();

    let token = "Bearer definetely invalid";

    let response = client
        .get(&get_me_url)
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_logout_success() {
    let address = spawn_server().await;

    let register_url = format!("http://{}/api/register", address);
    let login_url = format!("http://{}/api/login", address);
    let logout_url = format!("http://{}/api/logout", address);
    let client = reqwest::Client::new();

    let email = "logout_success@test.com";
    let body = serde_json::json!({
        "email": email,
        "password": "12345678"
    });

    let _ = client.post(&register_url).json(&body).send().await;

    let response: GenericResponse<AccessTokenData> = client
        .post(&login_url)
        .json(&body)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let token = response.data.unwrap().access_token;

    let response: GenericResponse<FilteredUser> = client
        .get(&logout_url)
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    clean_up_db(|db| async move {
        db.execute(sqlx::query!("DELETE FROM users WHERE email = $1", email))
            .await
            .unwrap();
    })
    .await;

    assert_eq!(response.status, Status::Success);
}

#[tokio::test]
async fn test_healthcheck() {
    let address = spawn_server().await;

    let url = format!("http://{}/api/healthcheck", address);

    let response = reqwest::get(url).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[cfg(test)]
async fn spawn_server() -> SocketAddr {
    let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    dotenv().ok();
    let config = Config::init();

    tokio::spawn(async move {
        run(listener, config).await.expect("Failed to run app");
    });

    address
}

#[cfg(test)]
async fn revoke_token_from_redis(access_token: &str) {
    use authentication_service::api::utils::jwt::verify_jwt;

    let config = Config::init();
    let access_token_uuid = verify_jwt(&config.access_token_public_key, access_token)
        .unwrap()
        .token_uuid;
    let mut redis_client = Client::open(config.redis_url.to_owned())
        .unwrap()
        .get_multiplexed_async_connection()
        .await
        .unwrap();
    let _: i64 = redis_client
        .del(access_token_uuid.to_string())
        .await
        .unwrap();
}

#[cfg(test)]
async fn clean_up_db<F, Fut>(query: F)
where
    F: Fn(Pool<Postgres>) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let config = Config::init();
    let db = connect_to_database(&config).await;
    query(db).await;
}

#[cfg(test)]
pub async fn connect_to_database(config: &Config) -> Pool<Postgres> {
    match PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => {
            println!("Enstablished db connection");
            pool
        }
        Err(err) => {
            println!("Failed to connecto to db with error: {:?}", err);
            std::process::exit(1)
        }
    }
}

#[cfg(test)]
#[derive(Debug, Deserialize)]
struct GenericResponse<T> {
    status: Status,
    data: Option<T>,
}

#[cfg(test)]
#[derive(Debug, Deserialize)]
struct AccessTokenData {
    access_token: String,
}
