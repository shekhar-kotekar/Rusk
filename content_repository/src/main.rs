use axum::{
    routing::{get, post},
    Router,
};
use std::{env, fmt::Debug};
use tokio::io::AsyncWriteExt;

use serde::Deserialize;

mod content_repository_manager;

#[tokio::main]
async fn main() {
    commons::enable_tracing();
    let config = read_config();

    let server_address = format!("0.0.0.0:{}", config.content_repository.server_port);
    let server = Router::new()
        .route("/health_check", get(health_check))
        .route("/write_data", post(write_data));

    let mut file_handle = content_repository_manager::init(config).await;

    tracing::info!("Starting rusk content repository on {}", server_address);

    let listener = tokio::net::TcpListener::bind(server_address).await.unwrap();
    axum::serve(listener, server).await.unwrap();

    file_handle
        .write_all("first line\n".as_bytes())
        .await
        .unwrap();

    file_handle
        .write_all("second line\n".as_bytes())
        .await
        .unwrap();

    file_handle.flush().await.unwrap();
    tracing::info!("Wrote to file");
}

async fn health_check() -> &'static str {
    "Content Repository Alive!"
}

async fn write_data() -> &'static str {
    "NOT IMPLEMENTED YET!"
}

fn read_config() -> Config {
    match env::var("CONFIG_FILE_PATH") {
        Ok(config_file_path) => {
            tracing::info!("Reading config from file: {}", config_file_path);
            let config_file = std::fs::read_to_string(config_file_path).unwrap();
            toml::from_str(&config_file).unwrap()
        }
        Err(_) => {
            panic!("CONFIG_FILE_PATH environment variable is not set");
        }
    }
}

#[derive(Debug, Deserialize)]
struct ContentRepositoryConfig {
    pub base_path: String,
    file_name_prefix: String,
    pub server_port: u16,
}

#[derive(Debug, Deserialize)]
struct Config {
    pub content_repository: ContentRepositoryConfig,
}
