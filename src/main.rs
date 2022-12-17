use aws_sdk_dynamodb::{model::AttributeValue, Client};
use lambda_http::{
    aws_lambda_events::serde_json, http::Method, run, service_fn, Body, Error, Request, Response,
};
use repositories::DatabaseRepository;
use services::{Document, DocumentService};
use tokio_stream::StreamExt;

mod config;
mod repositories;
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

pub async fn list_tables(client: &Client) -> Result<Vec<String>, Error> {
    let paginator = client.list_tables().into_paginator().items().send();
    let table_names = paginator.collect::<Result<Vec<_>, _>>().await?;
    Ok(table_names)
}

async fn handler(event: Request) -> Result<Response<Body>, Error> {
    let config = config::load_config().await;
    let client = Client::new(&config);
    let database_repository = DatabaseRepository::from_client(client);
    let (parts, _) = event.into_parts();
    let path = parts.uri.path();
    let method = parts.method;

    let response = match path {
        "/api/notes/documents" => match method {
            Method::GET => {
                let document_service = DocumentService::new(database_repository);
                let documents: Vec<Document> = document_service.list_all().await;

                let body = serde_json::to_string(&documents)?;

                Response::builder()
                    .status(200)
                    .header("content-type", "application/json")
                    .body(body.into())
                    .unwrap()
            }
            _ => Response::builder().status(405).body(Body::Empty).unwrap(),
        },
        _ => Response::builder().status(404).body(Body::Empty).unwrap(),
    };

    Ok(response)
}
