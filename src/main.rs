use anyhow::Result;
use authentication_service::{application::run, helper::config::Config};
use dotenv::dotenv;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenv().ok();

    let config = Config::init();
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    run(listener, config).await?;
    Ok(())
}
