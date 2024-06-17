use authentication_service::{
    api::utils::status::Status,
    application::{connect_to_database, run},
    helper::config::Config,
    model::user::User,
};
use dotenv::dotenv;
use serde::Deserialize;
use sqlx::{Executor, Pool, Postgres};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::test]
async fn test_register() {
    let address = spawn_server().await;

    let url = format!("http://{}/api/register", address);
    let client = reqwest::Client::new();

    let email = "email@test.com";
    let body = serde_json::json!({
        "email": email,
        "password": "12345678"
    });

    let response: GenericResponse<UserData> = client
        .post(&url)
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

    assert_eq!(response.data.unwrap().user.email, email);
}

#[tokio::test]
async fn test_healthcheck() {
    let address = spawn_server().await;

    let url = format!("http://{}/api/healthcheck", address);

    let response: GenericResponse<UserData> =
        reqwest::get(url).await.unwrap().json().await.unwrap();

    assert_eq!(response.status, Status::Success);
}

#[cfg(test)]
async fn spawn_server() -> SocketAddr {
    let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    dotenv().ok();
    let config = Config::init();

    tokio::spawn(async move {
        run(listener, config).await;
    });

    address
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
#[derive(Debug, Deserialize)]
struct GenericResponse<T> {
    status: Status,
    data: Option<T>,
}

#[cfg(test)]
#[derive(Debug, Deserialize)]
struct UserData {
    user: User,
}
