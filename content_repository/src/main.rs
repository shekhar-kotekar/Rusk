use bytes::Bytes;
use models::Command;
use tokio::{
    io::{AsyncReadExt, AsyncSeekExt},
    net::TcpListener,
    sync::{mpsc, oneshot},
};

mod content_repository_manager;
mod models;

#[tokio::main]
async fn main() {
    commons::enable_tracing();

    let config = commons::get_config().content_repository;

    let server_address = format!("0.0.0.0:{}", config.server_port);

    let mut file_handle = content_repository_manager::init(config).await;
    let (tx, mut rx) = mpsc::channel::<Command>(1000);

    tracing::info!("Rusk content repository listening on {}", server_address);
    let listener = TcpListener::bind(server_address).await.unwrap();
    tokio::spawn(async move {
        while let Some(data) = rx.recv().await {
            process_data(data, &mut file_handle).await;
        }
    });

    loop {
        let tx_clone = tx.clone();
        handle_client_request(tx_clone, &listener).await;
        // let (mut socket, _) = listener.accept().await.unwrap();
        // tokio::spawn(async move {
        //     let mut buf = vec![0; 1024];
        //     loop {
        //         let number_of_bytes_read = socket.read_buf(&mut buf).await.unwrap();
        //         if number_of_bytes_read == 0 {
        //             tracing::info!("0 bytes read, connection closed.");
        //             break;
        //         }
        //         let (tx, rx) = oneshot::channel::<u64>();
        //         let command = Command::Data {
        //             content: Bytes::copy_from_slice(&buf[..number_of_bytes_read]),
        //             tx,
        //         };
        //         tx_clone.send(command).await.unwrap();
        //         let response = rx.await.unwrap();
        //         tracing::info!("Response from content repo: {:?}", response);
        //     }
        // });
    }
}

async fn process_data(data: Command, file_handle: &mut tokio::fs::File) {
    match data {
        Command::Data { content, tx } => {
            let current_offset = file_handle.stream_position().await.unwrap();
            content_repository_manager::append_data(file_handle, &content).await;
            let _ = tx.send(current_offset);
        }
    }
}

async fn handle_client_request(
    tx_clone: mpsc::Sender<Command>,
    listener: &tokio::net::TcpListener,
) {
    let (mut socket, _) = listener.accept().await.unwrap();
    tokio::spawn(async move {
        let mut buf = vec![0; 1024];
        let (mut reader, writer) = socket.split();
        loop {
            let number_of_bytes_read = reader.read_buf(&mut buf).await.unwrap();
            if number_of_bytes_read == 0 {
                tracing::info!("0 bytes read, connection closed.");
                break;
            }
            let (tx, rx) = oneshot::channel::<u64>();
            let command = Command::Data {
                content: Bytes::copy_from_slice(&buf[..number_of_bytes_read]),
                tx,
            };
            tx_clone.send(command).await.unwrap();
            let response = rx.await.unwrap();
            tracing::info!("Response from content repo: {:?}", response);
        }
    });
}

#[cfg(test)]
mod tests {
    use commons::ContentRepositoryConfig;
    use tempfile::tempdir;

    use super::*;
    use std::{fs::File as StdFile, io::Read};

    #[tokio::test]
    async fn test_process_data() {
        let temp_dir = tempdir().unwrap();
        let test_config = ContentRepositoryConfig {
            base_path: temp_dir.path().to_str().unwrap().to_string(),
            file_name_prefix: String::from("test_wal_process_data"),
            server_port: 8080,
        };
        let mut file_handle = content_repository_manager::init(test_config).await;
        let (tx, rx) = oneshot::channel::<u64>();
        let data = Command::Data {
            content: Bytes::from_static(b"test_data"),
            tx,
        };
        process_data(data, &mut file_handle).await;

        let test_file_path = temp_dir.path().join("test_wal_process_data.txt");
        let mut file = StdFile::open(test_file_path.to_str().unwrap()).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, "test_data\n");
        let response_from_process_data = rx.await.unwrap();
        assert_eq!(response_from_process_data, 0);
    }

    #[tokio::test]
    async fn test_handle_client_request() {
        let (tx, mut rx) = mpsc::channel::<Command>(1000);
        let listener = tokio::net::TcpListener::bind("0.0.0.0:1234").await.unwrap();
        handle_client_request(tx.clone(), &listener).await;
    }
}
