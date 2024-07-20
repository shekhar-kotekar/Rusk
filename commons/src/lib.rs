use serde::Deserialize;
use std::{env, fmt::Debug};
use tracing_subscriber::fmt::format::FmtSpan;

pub fn enable_tracing() {
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

pub fn get_config() -> Config {
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

#[derive(Debug, Deserialize, Clone)]
pub struct ContentRepositoryConfig {
    pub base_path: String,
    pub file_name_prefix: String,
    pub server_port: u16,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub content_repository: ContentRepositoryConfig,
}
