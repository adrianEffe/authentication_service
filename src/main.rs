use authentication_service::{app::run, helper::config::Config};
use dotenv::dotenv;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let config = Config::init();
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    run(listener, config).await;
}
