use axum::{extract::State, Json};
use http::StatusCode;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::{processors::models::ProcessorCommand, AppState};

use super::models::{ClusterInfo, ProcessorConnectionRequest, ProcessorInfo};

#[tracing::instrument]
pub async fn is_alive() -> &'static str {
    "I am alive!"
}

#[tracing::instrument]
pub async fn get_cluster_info(
    State(server_state): State<AppState>,
) -> Result<Json<ClusterInfo>, StatusCode> {
    let mut processors_in_cluster: Vec<ProcessorInfo> = vec![];
    for processor_tx in server_state.parent_processor_tx.lock().await.values() {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        let command = ProcessorCommand::GetInfo { resp: resp_tx };
        processor_tx.send(command).await.unwrap();
        processors_in_cluster.push(resp_rx.await.unwrap());
    }

    let cluster_info = ClusterInfo {
        cluster_name: "Rusk Default Cluster".to_string(),
        processors: processors_in_cluster,
    };

    Ok(Json(cluster_info))
}

#[tracing::instrument]
pub async fn connect_processors(
    State(server_state): State<AppState>,
    Json(payload): Json<ProcessorConnectionRequest>,
) -> Result<Json<ProcessorInfo>, StatusCode> {
    let source_processor_id = Uuid::parse_str(&payload.source_processor_id).unwrap();
    let destination_processor_id = Uuid::parse_str(&payload.destination_processor_id).unwrap();

    match server_state
        .peers_tx
        .lock()
        .await
        .get(&destination_processor_id)
    {
        Some(tx) => {
            match server_state
                .parent_processor_tx
                .lock()
                .await
                .get(&source_processor_id)
            {
                Some(source_tx) => {
                    let (oneshot_tx, oneshot_rx) = oneshot::channel();
                    let command = ProcessorCommand::Connect {
                        destination_processor_id,
                        destination_processor_tx: tx.clone(),
                        resp: oneshot_tx,
                    };

                    source_tx.send(command).await.unwrap();
                    let processor_current_status = oneshot_rx.await.unwrap();
                    let result = Json(ProcessorInfo {
                        processor_id: source_processor_id.to_string(),
                        status: processor_current_status,
                        packets_processed_count: 0,
                    });
                    return Ok(result);
                }
                None => {
                    return Err(StatusCode::NOT_FOUND);
                }
            }
        }
        None => {
            return Err(StatusCode::NOT_FOUND);
        }
    }
}

#[tracing::instrument]
pub async fn disconnect_processors(
    State(server_state): State<AppState>,
    Json(payload): Json<ProcessorConnectionRequest>,
) -> Result<Json<ProcessorInfo>, StatusCode> {
    let source_processor_id = Uuid::parse_str(&payload.source_processor_id).unwrap();
    let destination_processor_id = Uuid::parse_str(&payload.destination_processor_id).unwrap();

    match server_state
        .parent_processor_tx
        .lock()
        .await
        .get(&source_processor_id)
    {
        Some(source_tx) => {
            let (oneshot_tx, oneshot_rx) = oneshot::channel();
            let command = ProcessorCommand::Disconnect {
                destination_processor_id,
                resp: oneshot_tx,
            };

            source_tx.send(command).await.unwrap();
            let processor_current_status = oneshot_rx.await.unwrap();
            let result = Json(ProcessorInfo {
                processor_id: source_processor_id.to_string(),
                status: processor_current_status,
                packets_processed_count: 0,
            });
            return Ok(result);
        }
        None => {
            return Err(StatusCode::NOT_FOUND);
        }
    }
}

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use axum::{routing::get, Router};
    use axum_test::TestServer;
    use commons::MainConfig;
    use std::collections::HashMap;
    use tokio_util::sync::CancellationToken;

    use tokio::sync::Mutex;

    use crate::{handlers::models::ClusterInfo, processors::models::ProcessorType};

    #[tokio::test]
    async fn test_is_alive() {
        let app = Router::new().route("/is_alive", get(super::is_alive));
        let test_server = TestServer::new(app).unwrap();
        let response = test_server.get("/is_alive").await;
        response.assert_status_ok();
        response.assert_text("I am alive!");
    }

    #[tokio::test]
    async fn test_get_cluster_info() {
        let config: MainConfig = MainConfig {
            server_port: 8080,
            processor_queue_length: 10,
        };

        let processor_mappings = HashMap::from([
            ("adder".to_string(), ProcessorType::SourceProcessor),
            ("doubler".to_string(), ProcessorType::Other),
        ]);
        let cancellation_token = CancellationToken::new();
        let state = super::AppState {
            config,
            cancellation_token: cancellation_token.clone(),
            peers_tx: Arc::new(Mutex::new(HashMap::new())),
            parent_processor_tx: Arc::new(Mutex::new(HashMap::new())),
            processor_types_mappings: Arc::new(Mutex::new(processor_mappings)),
        };
        let app = Router::new()
            .route("/get_cluster_info", get(super::get_cluster_info))
            .with_state(state);

        let test_server = TestServer::new(app).unwrap();
        let response = test_server.get("/get_cluster_info").await;
        response.assert_status_ok();

        let actual_cluster_details = response.json::<ClusterInfo>();
        let expected_cluster_details = ClusterInfo {
            cluster_name: "Rusk Default Cluster".to_string(),
            processors: vec![],
        };
        assert_eq!(actual_cluster_details, expected_cluster_details);
    }
}
