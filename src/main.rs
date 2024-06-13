use authentication_service::{app::app, helper::config::Config};
use dotenv::dotenv;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let config = Config::init();
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app(config).await).await.unwrap();
}
