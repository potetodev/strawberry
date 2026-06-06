use aws_sdk_s3::{Client, config::Region};
use poem::{
    Body, IntoResponse, Response, Route, Server, get, handler, listener::TcpListener, web::Path,
};

#[handler]
async fn fetch_file(Path(file_path): Path<String>) -> impl IntoResponse {
    let bucket = std::env::var("AWS_S3_BUCKET_NAME").expect("AWS_S3_BUCKET_NAME not set");
    let region_name =
        std::env::var("AWS_DEFAULT_REGION").unwrap_or_else(|_| "us-east-1".to_string());
    let bucket_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(Region::new(region_name))
        .load()
        .await;

    let client = Client::new(&bucket_config);

    match client
        .get_object()
        .bucket(&bucket)
        .key(&file_path)
        .send()
        .await
    {
        Ok(response) => Response::builder()
            .status(poem::http::StatusCode::OK)
            .body(Body::from_async_read(response.body.into_async_read())),
        Err(_) => Response::builder()
            .status(poem::http::StatusCode::NOT_FOUND)
            .body("File not found"),
    }
}

#[tokio::main]
async fn main() {
    let app = Route::new().at("/*file_path", get(fetch_file));

    Server::new(TcpListener::bind("127.0.0.1:8000"))
        .run(app)
        .await
        .unwrap();
}
