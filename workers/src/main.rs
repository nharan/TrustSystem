use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod pipeline;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("workers service started");
    let api_base = std::env::var("API_BASE").unwrap_or_else(|_| "http://localhost:8080".to_string());
    // One-shot worker to avoid hanging long-running process during guided runs
    let _ = pipeline::run_once(&api_base).await;
}



