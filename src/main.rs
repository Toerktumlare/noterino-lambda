use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::{Client, Endpoint};
use lambda_http::{
    aws_lambda_events::serde_json, http::Uri, run, service_fn, Body, Error, Request, Response,
};
use serde::Serialize;
use tokio_stream::StreamExt;

#[derive(Serialize)]
pub struct TableResponse {
    table_names: Vec<String>,
}

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

async fn handler(_event: Request) -> Result<Response<Body>, Error> {
    let _region_provider = RegionProviderChain::default_provider().or_else("us-north-1");
    let config = aws_config::from_env()
        .endpoint_resolver(Endpoint::immutable(Uri::from_static(
            "http://172.17.0.1:8000",
        )))
        .load()
        .await;
    let client = Client::new(&config);

    let tables = list_tables(&client).await?;

    let table_response = TableResponse {
        table_names: tables,
    };

    let body = serde_json::to_string(&table_response)?;

    let response = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(body.into())
        .unwrap();
    Ok(response)
}
