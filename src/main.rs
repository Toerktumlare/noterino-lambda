use aws_sdk_dynamodb::{model::AttributeValue, Client};
use lambda_http::{aws_lambda_events::serde_json, run, service_fn, Body, Error, Request, Response};
use serde::Serialize;
use tokio_stream::StreamExt;

mod config;

#[derive(Serialize)]
pub struct DocumentEntity {
    title: String,
    description: String,
    created: u32,
    #[serde(rename(serialize = "updatedBy"))]
    updated_by: String,
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
    let (parts, _) = _event.into_parts();
    let path = parts.uri.path();

    let response = match path {
        "/api/notes/documents" => {
            let req = client
                .query()
                .table_name(&table_name)
                .key_condition_expression("PK = :hashKey")
                .expression_attribute_values(":hashKey", AttributeValue::S("document".to_string()))
                .send()
                .await
                .unwrap();

            let items = req.items().unwrap();

            let documents: Vec<DocumentEntity> = items
                .iter()
                .map(|item| {
                    dbg!(&item["title"]);
                    let title = get_string(&item["title"]);
                    let description = get_string(&item["description"]);
                    let updated_by = get_string(&item["updatedBy"]);
                    let created = get_number(&item["created"]);
                    DocumentEntity {
                        title,
                        description,
                        created,
                        updated_by,
                    }
                })
                .collect();
            let body = serde_json::to_string(&documents)?;

            Response::builder()
                .status(200)
                .header("content-type", "application/json")
                .body(body.into())
                .unwrap()
        }
        _ => Response::builder().status(404).body(Body::Empty).unwrap(),
    };

    Ok(response)
}

pub fn get_string(v: &AttributeValue) -> String {
    if let AttributeValue::S(value) = v {
        value.to_owned()
    } else {
        "".to_string()
    }
}

pub fn get_number(v: &AttributeValue) -> u32 {
    if let AttributeValue::N(value) = v {
        value.parse().unwrap()
    } else {
        0
    }
}
