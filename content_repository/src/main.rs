use std::{
    env,
    fmt::Debug,
    fs::{self},
};

use serde::Deserialize;
use tokio::{
    fs::{File, OpenOptions},
    io::AsyncWriteExt,
};

#[tokio::main]
async fn main() {
    commons::enable_tracing();
    let config = read_config();

    let mut file_handle = init(config).await;

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

async fn init(config: Config) -> File {
    fs::create_dir_all(&config.content_repository.base_path).expect(
        format!(
            "Failed to create content repository directory: {}",
            config.content_repository.base_path
        )
        .as_str(),
    );
    let base_path = config.content_repository.base_path;
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(base_path + String::from("/file.txt").as_str())
        .await
        .unwrap()
}

fn read_config() -> Config {
    match env::var("CONFIG_FILE_PATH") {
        Ok(config_file_path) => {
            tracing::info!("Reading config from file: {}", config_file_path);
            let config_file = fs::read_to_string(config_file_path).unwrap();
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
}

#[derive(Debug, Deserialize)]
struct Config {
    pub content_repository: ContentRepositoryConfig,
}
