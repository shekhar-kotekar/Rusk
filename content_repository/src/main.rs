use std::{env, fmt::Debug, fs};

use serde::Deserialize;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

#[tokio::main]
async fn main() {
    commons::enable_tracing();
    let config = read_config();
    tracing::info!("Config: {:?}", config);

    fs::create_dir_all(&config.content_repository.base_path).expect(
        format!(
            "Failed to create content repository directory: {}",
            config.content_repository.base_path
        )
        .as_str(),
    );

    let mut file_handle = OpenOptions::new()
        .create(true)
        .append(true)
        .open(config.content_repository.base_path + "/file.txt")
        .await
        .unwrap();

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

fn read_config() -> Config {
    let config_file_path = env::var("CONFIG_FILE_PATH").expect("CONFIG_FILE_PATH not set");
    let config_file = fs::read_to_string(config_file_path).unwrap();
    toml::from_str(&config_file).unwrap()
}

#[derive(Debug, Deserialize)]
struct ContentRepositoryConfig {
    pub base_path: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    pub content_repository: ContentRepositoryConfig,
}
