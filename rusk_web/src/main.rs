use axum::{
    routing::{delete, get, post},
    Router,
};
use http::{header, Method};
use tokio::signal;
use tower_http::cors::{Any, CorsLayer};

const SERVER_PORT: &str = "5056";

#[tokio::main]
async fn main() {
    commons::enable_tracing();
    let server_address = format!("0.0.0.0:{}", SERVER_PORT);

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE])
        .allow_origin(Any);

    let server = Router::new()
        .route("/is_alive", get(is_alive))
        .route("/processor/create", post(create_processor))
        .route("/processor/delete", delete(delete_processor))
        .route("/processor/stop", post(stop_processor))
        .route("/processor/start", post(start_processor))
        .route("/processor/get_status", get(get_processor_status))
        .route("/processor/get_info", get(get_processor_info))
        .route("/processor/connect", post(connect_processors))
        .route("/processor/disconnect", post(disconnect_processors))
        .layer(cors);

    tracing::info!("Starting rusk web server on {}", server_address);

    let listener = tokio::net::TcpListener::bind(server_address).await.unwrap();
    axum::serve(listener, server)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl-C signal, shutting down...");
        },
        _ = terminate => {
            tracing::info!("Received terminate signal, shutting down...");
        },
    }
}

async fn is_alive() -> &'static str {
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

async fn get_processor_info() -> &'static str {
    "get processor info"
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
    async fn test_is_alive() {
        let app = Router::new().route("/is_alive", get(super::is_alive));
        let test_server = TestServer::new(app).unwrap();
        let response = test_server.get("/is_alive").await;
        response.assert_status_ok();
        response.assert_text("Alive");
    }

    #[tokio::test]
    async fn test_get_processor_info() {
        let route = "/processor/get_info";
        let app = Router::new().route(&route, get(super::get_processor_info));
        let test_server = TestServer::new(app).unwrap();
        let response = test_server.get(&route).await;
        response.assert_status_ok();
        response.assert_text("get processor info");
    }
}
