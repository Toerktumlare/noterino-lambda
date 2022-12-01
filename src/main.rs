use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::Client;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use log::{error, info};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Request {}

#[derive(Debug, Serialize)]
struct SuccessResponse {
    pub body: String,
}

#[derive(Debug, Serialize)]
struct FailureResponse {
    pub body: String,
}

impl std::fmt::Display for FailureResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.body)
    }
}

impl std::error::Error for FailureResponse {}

type Response = Result<SuccessResponse, FailureResponse>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    let func = service_fn(handler);
    lambda_runtime::run(func).await?;

    Ok(())
}

async fn handler(_event: LambdaEvent<Request>) -> Response {
    let region_provider = RegionProviderChain::default_provider().or_else("eu-north-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&config);

    let response = client.list_tables().send().await.map_err(|err| {
        error!("Failed to fetch tables, error: {err}");
        FailureResponse {
            body: "Could not fetch tables".to_owned(),
        }
    })?;

    info!("Tables:");

    let names = response.table_names().unwrap_or_default();

    for name in names {
        info!("   {name}");
    }

    info!("Found {} tables", names.len());

    Ok(SuccessResponse {
        body: format!("Number of tables found: {}", names.len()),
    })
}
