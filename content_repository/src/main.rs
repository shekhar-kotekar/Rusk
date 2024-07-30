use bytes::Bytes;
use models::Command;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
    net::TcpListener,
    select,
    sync::{mpsc, oneshot},
};
use tokio_util::sync::CancellationToken;

mod content_repository_manager;
mod models;

#[tokio::main]
async fn main() {
    commons::enable_tracing();

    let config = commons::get_config().content_repository;

    let server_port = config.server_port;
    let server_address = format!("0.0.0.0:{}", server_port);
    let listener = TcpListener::bind(&server_address).await.unwrap();
    tracing::info!("Rusk content repository listening on {}", server_address);

    let mut file_handle = content_repository_manager::init(config).await;

    let (conten_repo_manager_tx, mut content_repo_manager_rx) = mpsc::channel::<Command>(1000);

    let cancellation_token = CancellationToken::new();
    let cancellation_token_for_content_repo_manager = cancellation_token.clone();

    tokio::spawn(async move {
        while let Some(data) = content_repo_manager_rx.recv().await {
            if cancellation_token_for_content_repo_manager.is_cancelled() {
                tracing::info!("Cancellation token received. Stopping content repo manager task.");
                break;
            }
            process_data(data, &mut file_handle).await;
        }
    });

    accept_client_connections(listener, conten_repo_manager_tx, cancellation_token.clone()).await;

    match tokio::signal::ctrl_c().await {
        Ok(_) => {
            tracing::info!("Ctrl-C signal received. Shutting down content repository server.");
            cancellation_token.cancel();
        }
        Err(e) => {
            tracing::error!("Failed to listen for Ctrl-C signal: {:?}", e);
        }
    }
}

async fn process_data(data: Command, file_handle: &mut tokio::fs::File) {
    match data {
        Command::Data {
            content,
            tx: oneshot_tx,
        } => {
            let record_start_offset = file_handle.stream_position().await.unwrap();
            content_repository_manager::append_data(file_handle, &content).await;
            let _ = oneshot_tx.send(record_start_offset);
        }
    }
}

async fn accept_client_connections(
    listener: TcpListener,
    conten_repo_manager_tx: mpsc::Sender<Command>,
    cancellation_token: CancellationToken,
) {
    loop {
        tracing::info!("Waiting for new client connection...");
        select! {
            _ = cancellation_token.cancelled() => {
                tracing::info!("Cancellation token received. Stopping client connection listener.");
                break;
            }
            Ok((mut socket, _)) = listener.accept() => {
                tracing::info!("New client connected: {:?}", socket.peer_addr().unwrap());

                let conten_repo_manager_tx_clone = conten_repo_manager_tx.clone();
                tokio::spawn(async move {
                    let (reader, mut writer) = socket.split();
                    // TODO: Do we need to send cancellation token here as well?
                    match handle_client_request(conten_repo_manager_tx_clone, reader).await {
                        Some(response) => {
                            tracing::info!("Response from content repo: {:?}", response);
                            writer.write_u64(response).await.unwrap();
                        }
                        None => {
                            tracing::info!("Client connection closed.");
                            writer.write_u64(0).await.unwrap();
                        }
                    }
                    writer.flush().await.unwrap();
                    writer.shutdown().await.unwrap();
                });
            }
        }
    }
}

async fn handle_client_request<Reader>(
    tx_clone: mpsc::Sender<Command>,
    mut reader: Reader,
) -> Option<u64>
where
    Reader: AsyncRead + Unpin,
{
    let mut buffer = vec![0; 1024];
    loop {
        if let Ok(bytes_read) = reader.read_buf(&mut buffer).await {
            if bytes_read == 0 {
                tracing::info!("0 bytes read, connection closed.");
                return None;
            }
            let (one_shot_tx, one_shot_rx) = oneshot::channel::<u64>();
            let command = Command::Data {
                content: Bytes::copy_from_slice(&buffer[..bytes_read]),
                tx: one_shot_tx,
            };
            tx_clone.send(command).await.unwrap();
            let response = one_shot_rx.await.unwrap();
            tracing::info!("Response from content repo: {:?}", response);
            return Some(response);
        }
    }
}

#[cfg(test)]
mod tests {
    use commons::ContentRepositoryConfig;
    use tempfile::tempdir;
    use tokio::{io::AsyncWriteExt, net::TcpStream};

    use super::*;
    use std::{fs::File as StdFile, io::Read};

    #[tokio::test]
    async fn test_accept_client_connections() {
        let listener = TcpListener::bind("127.0.0.1:5056").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let cancellation_token = CancellationToken::new();
        let cancellation_token_clone = cancellation_token.clone();
        let (tx, _) = mpsc::channel::<Command>(10);

        tokio::spawn(async move {
            accept_client_connections(listener, tx, cancellation_token_clone).await;
        });

        let mut stream = TcpStream::connect(addr).await.unwrap();
        stream.write_all(b"data_to_write").await.unwrap();

        cancellation_token.cancel();
    }
}
