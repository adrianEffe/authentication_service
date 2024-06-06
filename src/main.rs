mod api;
use api::healthcheck::healthcheck;
use axum::{routing::get, Router};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app()).await.unwrap();
}

fn app() -> Router {
    Router::new().route("/api/healthcheck", get(healthcheck))
}

#[tokio::test]
async fn test_healthcheck() {
    let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app()).await.unwrap();
    });

    let url = format!("http://{}/api/healthcheck", address);

    let body = reqwest::get(url).await.unwrap().text().await.unwrap();

    assert_eq!(body, "ioafudsio");

    println!("body = {body:?}");
}
