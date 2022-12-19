use aws_sdk_dynamodb::Client;
use lambda_http::{http::Method, run, service_fn, Body, Error, Request, Response};
use nanoserde::SerJson;
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
    let path = path.split('/').last().unwrap();
    let method = parts.method;
    dbg!(path);

    let response = match path {
        "" => match method {
            Method::GET => {
                let document_service = DocumentService::new(database_repository);
                let documents: Vec<Document> = document_service.list_all().await;

                let body = SerJson::serialize_json(&documents);

                Response::builder()
                    .status(200)
                    .header("content-type", "application/json")
                    .body(body.into())
                    .unwrap()
            }
            _ => Response::builder().status(405).body(Body::Empty).unwrap(),
        },
        id if !id.is_empty() => match method {
            Method::GET => {
                let document_service = DocumentService::new(database_repository);
                let document = document_service.fetch_by_id(id).await;

                let body = SerJson::serialize_json(&document);
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
