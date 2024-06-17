use authentication_service::{
    api::utils::status::Status,
    application::{connect_to_database, run},
    helper::config::Config,
    model::user::User,
};
use dotenv::dotenv;
use serde::Deserialize;
use tokio::net::TcpListener;

#[tokio::test]
async fn test_register() {
    let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    dotenv().ok();
    let config = Config::init();

    let db = connect_to_database(&config.clone()).await;

    tokio::spawn(async move {
        run(listener, config).await;
    });

    let url = format!("http://{}/api/register", address);

    let response: GenericResponse<UserData> =
        reqwest::get(url).await.unwrap().json().await.unwrap();

    assert_eq!(response.data.unwrap().user.email, "adrian@email.com");

    let _ = sqlx::query!(
        "DELETE FROM users WHERE email = $1",
        "adrian@email.com".to_string()
    )
    .fetch_one(&db)
    .await;
}

#[tokio::test]
async fn test_healthcheck() {
    let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    dotenv().ok();
    let config = Config::init();

    tokio::spawn(async move {
        run(listener, config).await;
    });

    let url = format!("http://{}/api/healthcheck", address);

    let response: GenericResponse<UserData> =
        reqwest::get(url).await.unwrap().json().await.unwrap();

    assert_eq!(response.status, Status::Success);
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
