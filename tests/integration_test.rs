use authentication_service::{api::utils::status::Status, app};
use serde::Deserialize;
use tokio::net::TcpListener;

#[tokio::test]
async fn test_healthcheck() {
    let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app()).await.unwrap();
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
