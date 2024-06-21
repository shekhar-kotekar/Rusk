use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    commons::enable_tracing();
    tracing::info!("Starting rusk web server");
    let server = Router::new().route("/", get(root));
    let listener = tokio::net::TcpListener::bind("").await.unwrap();
    axum::serve(listener, server).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}
