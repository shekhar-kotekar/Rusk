use axum::{
    routing::{get, post},
    Router,
};

const SERVER_PORT: &str = "5056";

#[tokio::main]
async fn main() {
    commons::enable_tracing();
    let server_address = format!("0.0.0.0:{}", SERVER_PORT);
    let server = Router::new()
        .route("/health_check", get(health_check))
        .route("/create_processor", post(create_processor))
        .route("/delete_processor", post(delete_processor))
        .route("/stop_processor", post(stop_processor))
        .route("/start_processor", post(start_processor))
        .route("/get_processor_status", get(get_processor_status))
        .route("/connect_processors", post(connect_processors))
        .route("/disconnect_processors", post(disconnect_processors));

    tracing::info!("Starting rusk web server on {}", server_address);

    let listener = tokio::net::TcpListener::bind(server_address).await.unwrap();
    axum::serve(listener, server).await.unwrap();
}

async fn health_check() -> &'static str {
    "Alive"
}

async fn create_processor() -> &'static str {
    "NOT IMPLEMENTED YET!"
}

async fn delete_processor() -> &'static str {
    "NOT IMPLEMENTED YET!"
}

async fn stop_processor() -> &'static str {
    "NOT IMPLEMENTED YET!"
}

async fn start_processor() -> &'static str {
    "NOT IMPLEMENTED YET!"
}

async fn get_processor_status() -> &'static str {
    "NOT IMPLEMENTED YET!"
}

async fn connect_processors() -> &'static str {
    "NOT IMPLEMENTED YET!"
}

async fn disconnect_processors() -> &'static str {
    "NOT IMPLEMENTED YET!"
}

#[cfg(test)]
mod rusk_web_server_tests {
    use axum::{routing::get, Router};
    use axum_test::TestServer;

    #[tokio::test]
    async fn test_health_check() {
        let app = Router::new().route("/health_check", get(super::health_check));
        let test_server = TestServer::new(app).unwrap();
        let response = test_server.get("/health_check").await;
        response.assert_status_ok();
        response.assert_text("Alive");
    }
}
