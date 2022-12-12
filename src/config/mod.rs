use aws_config::SdkConfig;
use aws_sdk_dynamodb::Endpoint;
use lambda_http::http::Uri;

pub async fn load_config() -> SdkConfig {
    aws_config::from_env()
        .endpoint_resolver(Endpoint::immutable(Uri::from_static(
            "http://172.17.0.1:8000",
        )))
        .load()
        .await
}
