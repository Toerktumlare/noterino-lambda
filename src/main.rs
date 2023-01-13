use aws_sdk_dynamodb::Client;
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use router::RouterDelegate;
use tokio_stream::StreamExt;

mod config;
mod repositories;
mod services;
mod router;

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

pub async fn list_tables(client: &Client) -> Result<Vec<String>, Error> {
    let paginator = client.list_tables().into_paginator().items().send();
    let table_names = paginator.collect::<Result<Vec<_>, _>>().await?;
    Ok(table_names)
}

async fn handler(event: Request) -> Result<Response<Body>, Error> {
    let config = config::load_config().await;
    let client = Client::new(&config);
    let (parts, _) = event.into_parts();
    let path = parts.uri.path();

    let router = RouterDelegate::new(client);
    let response = router.handle(&parts).await;

    Ok(response)
}
