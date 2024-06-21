use axum::{routing::get, Router};

const SERVER_PORT: &str = "5056";

#[tokio::main]
async fn main() {
    commons::enable_tracing();
    let server_address = format!("localhsot:{}", SERVER_PORT);
    tracing::info!("Starting rusk web server on: {}", server_address);
    let server = Router::new().route("/", get(root));
    let listener = tokio::net::TcpListener::bind(server_address).await.unwrap();
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
