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

#[cfg(test)]
mod tests {
    use axum::{routing::get, Router};
    use axum_test::TestServer;

    #[tokio::test]
    async fn test_server() {
        let app = Router::new().route("/", get(super::root));
        let test_server = TestServer::new(app).unwrap();
        let response = test_server.get("/").await;
        response.assert_status_ok();
        response.assert_text("Hello, World!");
    }
}
