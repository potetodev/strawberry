use dotenv::dotenv;
use std::time::Duration;

use aws_sdk_s3::{Client, config::Region, presigning::PresigningConfig};
use poem::{
    EndpointExt, IntoResponse, Result, Route, Server,
    error::{InternalServerError, NotFoundError},
    get, handler,
    listener::TcpListener,
    web::{Data, Path, Redirect},
};

#[handler]
async fn create_presigned_url(
    client: Data<&Client>,
    Path(file_name): Path<String>,
) -> Result<impl IntoResponse> {
    let expire = Duration::from_secs(2 * 60 * 60);
    let config = PresigningConfig::expires_in(expire).map_err(|e| InternalServerError(e))?;

    let bucket = std::env::var("AWS_S3_BUCKET_NAME").map_err(|e| InternalServerError(e))?;

    let presigned = client
        .get_object()
        .bucket(&bucket)
        .key(&file_name)
        .presigned(config)
        .await
        .map_err(|_| NotFoundError)?;

    let url = presigned.uri().to_string();
    Ok(Redirect::temporary(url))
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let region_name =
        std::env::var("AWS_DEFAULT_REGION").unwrap_or_else(|_| "us-east-1".to_string());
    let bucket_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(Region::new(region_name))
        .load()
        .await;

    let client = Client::new(&bucket_config);
    let app = Route::new()
        .at("/*file_path", get(create_presigned_url))
        .data(client);
    let address = format!(
        "0.0.0.0:{}",
        std::env::var("PORT").unwrap_or("8000".to_string())
    );

    Server::new(TcpListener::bind(address))
        .run(app)
        .await
        .unwrap();
}
