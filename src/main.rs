use aws_sdk_dynamodb::Client;
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use router::RouterDelegate;

mod config;
mod controllers;
mod repositories;
mod router;
mod services;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();
    run(service_fn(handler)).await?;
    Ok(())
}

async fn handler(event: Request) -> Result<Response<Body>, Error> {
    let config = config::load_config().await;
    let client = Client::new(&config);

    let router = RouterDelegate::new(client);
    let response = router.handle(event).await;

    Ok(response)
}
