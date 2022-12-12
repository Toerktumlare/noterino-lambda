use aws_sdk_dynamodb::{model::AttributeValue, Client};
use lambda_http::{aws_lambda_events::serde_json, run, service_fn, Body, Error, Request, Response};
use serde::Serialize;
use tokio_stream::StreamExt;
use tracing::info;

mod config;

#[derive(Serialize)]
pub struct TableResponse {
    size: i32,
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
    let config = config::load_config().await;
    let table_name =
        std::env::var("TABLE_NAME").expect("A TABLE_NAME must be set in this app's Lambda");
    let client = Client::new(&config);
    info!("Table name: ");
    info!(table_name);

    let values = client
        .query()
        .table_name("notes")
        .consistent_read(true)
        .key_condition_expression("PK = :hashKey")
        .expression_attribute_values(":hashKey", AttributeValue::S("document".to_string()))
        .send()
        .await
        .unwrap();

    let size = values.count();

    let table_response = TableResponse { size };

    let body = serde_json::to_string(&table_response)?;

    let response = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(body.into())
        .unwrap();
    Ok(response)
}
