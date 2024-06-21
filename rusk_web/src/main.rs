use axum::{routing::get, Router};
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() {
    enable_tracing();
    tracing::info!("Starting rusk web server");
    let server = Router::new().route("/", get(root));
    let listener = tokio::net::TcpListener::bind("").await.unwrap();
    axum::serve(listener, server).await.unwrap();
}

fn enable_tracing() {
    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
        .with_thread_ids(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    tracing::info!("Tracing enabled!");
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
