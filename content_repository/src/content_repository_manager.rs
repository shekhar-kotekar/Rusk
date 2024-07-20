use commons::ContentRepositoryConfig;
use tokio::{
    fs::{File, OpenOptions},
    io::AsyncWriteExt,
};

pub async fn init(config: ContentRepositoryConfig) -> File {
    tracing::info!("Initializing content repository");
    std::fs::create_dir_all(&config.base_path).expect(
        format!(
            "Failed to create content repository directory: {}",
            config.base_path
        )
        .as_str(),
    );
    tracing::info!("Content repository directory created: {}", config.base_path);
    let file_path = format!("{}/{}.txt", config.base_path, config.file_name_prefix);
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
        .await
        .unwrap()
}

pub async fn append_data(file_handle: &mut File, data: &[u8]) {
    file_handle.write_all(data).await.unwrap();
    file_handle.write_all("\n".as_bytes()).await.unwrap();
    file_handle.flush().await.unwrap();
}

#[cfg(test)]
mod tests {
    use commons::ContentRepositoryConfig;
    use tempfile::tempdir;

    use super::*;
    use std::{fs::File as StdFile, io::Read};

    #[tokio::test]
    async fn test_init() {
        let test_config = ContentRepositoryConfig {
            base_path: String::from("/tmp"),
            file_name_prefix: String::from("test_wal_init"),
            server_port: 8080,
        };
        let file_handle = init(test_config).await;
        let file = StdFile::open("/tmp/test_wal_init.txt").unwrap();
        assert_eq!(
            file.metadata().unwrap().len(),
            file_handle.metadata().await.unwrap().len()
        );
    }

    #[tokio::test]
    async fn test_append_data() {
        let temp_dir = tempdir().unwrap();
        let test_config = ContentRepositoryConfig {
            base_path: temp_dir.path().to_str().unwrap().to_string(),
            file_name_prefix: String::from("test_wal_append"),
            server_port: 8080,
        };
        let mut file_handle = init(test_config).await;

        append_data(&mut file_handle, "test line 1".as_bytes()).await;
        append_data(&mut file_handle, "test line 2".as_bytes()).await;

        let test_file_path = temp_dir.path().join("test_wal_append.txt");
        let mut file = StdFile::open(test_file_path.to_str().unwrap()).unwrap();
        let mut actual_contents = String::new();
        file.read_to_string(&mut actual_contents).unwrap();

        let expected_contents = "test line 1\ntest line 2\n";
        assert_eq!(actual_contents, expected_contents);
    }
}
