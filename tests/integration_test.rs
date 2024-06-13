use authentication_service::{
    api::utils::status::Status,
    application::{app, connect_to_database, AppState},
    helper::config::Config,
};
use dotenv::dotenv;
use serde::Deserialize;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::test]
async fn test_healthcheck() {
    let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    dotenv().ok();
    let config = Config::init();
    let pool = connect_to_database(&config).await;

    let app_state = Arc::new(AppState { db: pool });
    let app = app(app_state);
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let url = format!("http://{}/api/healthcheck", address);

    let body = reqwest::get(url).await.unwrap().text().await.unwrap();
    let deserialised: StatusResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(deserialised.status, Status::Success);
}

#[cfg(test)]
#[derive(Deserialize)]
struct StatusResponse {
    status: Status,
}
