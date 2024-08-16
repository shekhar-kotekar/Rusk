use std::collections::HashMap;

use crate::{
    adder_func, doubler_func,
    processors::{
        base_processor::{SinkProcessor, SourceProcessor},
        in_memory_processor::InMemoryProcessor,
        in_memory_source_processor::InMemorySourceProcessor,
        models::{Message, ProcessorCommand, ProcessorStatus, ProcessorType},
    },
    AppState,
};
use axum::{debug_handler, extract::Path, extract::State, Json};
use http::StatusCode;
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::models::{ProcessorConnectionRequest, ProcessorInfo, RequestDetails, ResponseDetails};

const PARENT_PROCESSOR_CHANNEL_SIZE: usize = 10;

fn create_source_processor(
    processor_name: String,
    processor_to_parent_rx: mpsc::Receiver<ProcessorCommand>,
    cancellation_token: CancellationToken,
) -> Uuid {
    let mut processor = InMemorySourceProcessor::new(
        processor_name,
        processor_to_parent_rx,
        HashMap::new(),
        cancellation_token,
    );

    let processor_id = processor.processor_id;

    tokio::spawn(async move {
        processor.run(adder_func).await;
    });
    processor_id
}

fn create_normal_processor(
    processor_name: String,
    parent_rx: mpsc::Receiver<ProcessorCommand>,
    peers_rx: mpsc::Receiver<Message>,
    cancellation_token: CancellationToken,
) -> Uuid {
    let mut processor =
        InMemoryProcessor::new(processor_name, peers_rx, parent_rx, cancellation_token);
    let processor_id = processor.processor_id;
    tokio::spawn(async move {
        processor.run(doubler_func).await;
    });
    processor_id
}

#[tracing::instrument]
pub async fn create_processor(
    State(server_state): State<AppState>,
    Json(payload): Json<RequestDetails>,
) -> Result<Json<ResponseDetails>, StatusCode> {
    let (parent_to_processor_tx, processor_to_parent_rx) =
        mpsc::channel::<ProcessorCommand>(PARENT_PROCESSOR_CHANNEL_SIZE);

    match server_state
        .processor_types_mappings
        .lock()
        .await
        .get(payload.processor_name.as_str())
    {
        Some(ProcessorType::SourceProcessor) => {
            let processor_id = create_source_processor(
                payload.processor_name.clone(),
                processor_to_parent_rx,
                server_state.cancellation_token.clone(),
            );

            server_state
                .parent_processor_tx
                .lock()
                .await
                .insert(processor_id, parent_to_processor_tx);

            let result = Json(ResponseDetails {
                processor_id: processor_id.to_string(),
                status: ProcessorStatus::Stopped,
            });
            return Ok(result);
        }
        Some(ProcessorType::Other) => {
            let (peers_tx, peers_rx) =
                mpsc::channel::<Message>(server_state.config.processor_queue_length);

            let processor_id = create_normal_processor(
                payload.processor_name.clone(),
                processor_to_parent_rx,
                peers_rx,
                server_state.cancellation_token.clone(),
            );

            server_state
                .parent_processor_tx
                .lock()
                .await
                .insert(processor_id, parent_to_processor_tx);

            server_state
                .peers_tx
                .lock()
                .await
                .insert(processor_id, peers_tx);

            let result = Json(ResponseDetails {
                processor_id: processor_id.to_string(),
                status: ProcessorStatus::Stopped,
            });
            return Ok(result);
        }
        None => {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
}

#[tracing::instrument]
pub async fn start_processor(
    State(server_state): State<AppState>,
    Json(payload): Json<RequestDetails>,
) -> Result<Json<ResponseDetails>, StatusCode> {
    let processor_id = Uuid::parse_str(&payload.processor_id.unwrap()).unwrap();

    match server_state
        .parent_processor_tx
        .lock()
        .await
        .get(&processor_id)
    {
        Some(tx) => {
            let (oneshot_tx, oneshot_rx) = oneshot::channel();
            let command = ProcessorCommand::Start { resp: oneshot_tx };

            tx.send(command).await.unwrap();
            match oneshot_rx.await.unwrap() {
                ProcessorStatus::Running => {
                    let result = Json(ResponseDetails {
                        processor_id: processor_id.to_string(),
                        status: ProcessorStatus::Running,
                    });
                    return Ok(result);
                }
                _ => {
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        }
        None => {
            tracing::error!("Processor not found: {}", processor_id);
            return Err(StatusCode::NOT_FOUND);
        }
    }
}

#[debug_handler]
#[tracing::instrument]
pub async fn stop_processor(
    State(server_state): State<AppState>,
    Json(payload): Json<RequestDetails>,
) -> Result<Json<ResponseDetails>, StatusCode> {
    let processor_id = Uuid::parse_str(&payload.processor_id.unwrap()).unwrap();
    match server_state
        .parent_processor_tx
        .lock()
        .await
        .get(&processor_id)
    {
        Some(tx) => {
            let (oneshot_tx, oneshot_rx) = oneshot::channel();
            let command = ProcessorCommand::Stop { resp: oneshot_tx };

            tx.send(command).await.unwrap();
            match oneshot_rx.await.unwrap() {
                ProcessorStatus::Stopped => {
                    let result = Json(ResponseDetails {
                        processor_id: processor_id.to_string(),
                        status: ProcessorStatus::Stopped,
                    });
                    return Ok(result);
                }
                _ => {
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        }
        None => {
            tracing::error!("Processor not found: {}", processor_id);
            return Err(StatusCode::NOT_FOUND);
        }
    }
}

#[tracing::instrument]
pub async fn delete_processor(State(server_state): State<AppState>) -> &'static str {
    "NOT IMPLEMENTED YET!"
}

#[tracing::instrument]
pub async fn get_status(
    State(server_state): State<AppState>,
    Json(payload): Json<RequestDetails>,
) -> Result<Json<ResponseDetails>, StatusCode> {
    // TODO: How to get processor status?
    let processor_id = Uuid::parse_str(&payload.processor_id.unwrap()).unwrap();
    println!("checking status of Processor ID: {}", processor_id);
    match server_state
        .parent_processor_tx
        .lock()
        .await
        .get(&processor_id)
    {
        Some(tx) => {
            let (oneshot_tx, oneshot_rx) = oneshot::channel();
            let command = ProcessorCommand::GetStatus { resp: oneshot_tx };

            tx.send(command).await.unwrap();

            let processor_current_status = oneshot_rx.await.unwrap();
            let result = Json(ResponseDetails {
                processor_id: processor_id.to_string(),
                status: processor_current_status,
            });
            return Ok(result);
        }
        None => {
            tracing::error!("Processor not found: {}", processor_id);
            return Err(StatusCode::NOT_FOUND);
        }
    }
}

#[tracing::instrument]
pub async fn get_processor_info(
    State(server_state): State<AppState>,
    Path(processor_id): Path<String>,
) -> Result<Json<ProcessorInfo>, StatusCode> {
    match server_state
        .parent_processor_tx
        .lock()
        .await
        .get(&Uuid::parse_str(&processor_id).unwrap())
    {
        Some(tx) => {
            let (oneshot_tx, oneshot_rx) = oneshot::channel();
            let command = ProcessorCommand::GetInfo { resp: oneshot_tx };

            tx.send(command).await.unwrap();
            let processor_info = oneshot_rx.await.unwrap();
            let result = Json(processor_info);
            return Ok(result);
        }
        None => {
            return Err(StatusCode::NOT_FOUND);
        }
    }
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
                        number_of_packets_processed: 0,
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
                number_of_packets_processed: 0,
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
    use std::{collections::HashMap, sync::Arc};

    use axum::{
        routing::{get, patch, post},
        Router,
    };
    use axum_test::TestServer;
    use commons::MainConfig;
    use serde_json::json;
    use tokio::sync::Mutex;
    use tokio_util::sync::CancellationToken;

    use crate::{
        handlers::models::{RequestDetails, ResponseDetails},
        processors::models::ProcessorType,
    };

    #[tokio::test]
    async fn test_create_processor() {
        let route = "/create_processor";

        let config: MainConfig = MainConfig {
            server_port: 8080,
            processor_queue_length: 10,
        };
        let cancellation_token = CancellationToken::new();
        let processor_mappings = HashMap::from([
            (
                "adder_processor".to_string(),
                ProcessorType::SourceProcessor,
            ),
            ("doubler_processor".to_string(), ProcessorType::Other),
        ]);

        let state = super::AppState {
            config,
            cancellation_token: cancellation_token.clone(),
            peers_tx: Arc::new(Mutex::new(HashMap::new())),
            parent_processor_tx: Arc::new(Mutex::new(HashMap::new())),
            processor_types_mappings: Arc::new(Mutex::new(processor_mappings)),
        };

        let app = Router::new()
            .route(&route, post(super::create_processor))
            .with_state(state);

        let test_server = TestServer::new(app).unwrap();
        let request_body: RequestDetails = RequestDetails {
            processor_name: "adder_processor".to_string(),
            processor_id: None,
        };

        let response = test_server.post(route).json(&json!(request_body)).await;
        response.assert_status_ok();

        let response_details = response.json::<ResponseDetails>();
        assert_eq!(response_details.status, super::ProcessorStatus::Stopped);

        cancellation_token.cancel();
    }

    #[tokio::test]
    async fn test_get_processor_status() {
        let create_processor_route = "/create_processor";
        let start_processor_route = "/start_processor";
        let stop_processor_route = "/stop_processor";
        let get_status_route = "/get_status";

        let config: MainConfig = MainConfig {
            server_port: 8080,
            processor_queue_length: 10,
        };
        let processor_mappings = HashMap::from([
            (
                "adder_processor".to_string(),
                ProcessorType::SourceProcessor,
            ),
            ("doubler_processor".to_string(), ProcessorType::Other),
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
            .route(&create_processor_route, post(super::create_processor))
            .route(&start_processor_route, patch(super::start_processor))
            .route(&stop_processor_route, patch(super::stop_processor))
            .route(&get_status_route, get(super::get_status))
            .with_state(state);

        let test_server = TestServer::new(app).unwrap();

        let create_processor_response = test_server
            .post(create_processor_route)
            .json(&json!(RequestDetails {
                processor_name: "adder_processor".to_string(),
                processor_id: None,
            }))
            .await;
        create_processor_response.assert_status_ok();
        let response_details = create_processor_response.json::<ResponseDetails>();
        assert_eq!(response_details.status, super::ProcessorStatus::Stopped);

        let request_body: RequestDetails = RequestDetails {
            processor_name: "adder_processor".to_string(),
            processor_id: Some(response_details.processor_id),
        };
        let get_status_response = test_server
            .get(get_status_route)
            .json(&json!(request_body))
            .await;
        get_status_response.assert_status_ok();
        let response_details = get_status_response.json::<ResponseDetails>();
        assert_eq!(
            response_details.status,
            super::ProcessorStatus::Stopped,
            "Processor status should be 'Stopped' as soon as we have created the processor."
        );

        let start_processor_response = test_server
            .patch(start_processor_route)
            .json(&json!(request_body))
            .await;
        start_processor_response.assert_status_ok();
        let response_details = start_processor_response.json::<ResponseDetails>();
        assert_eq!(response_details.status, super::ProcessorStatus::Running);
        let get_status_response = test_server
            .get(get_status_route)
            .json(&json!(request_body))
            .await;
        get_status_response.assert_status_ok();
        let response_details = get_status_response.json::<ResponseDetails>();
        assert_eq!(
            response_details.status,
            super::ProcessorStatus::Running,
            "Processor status should be 'Running' after we start the processor."
        );

        let stop_processor_response = test_server
            .patch(stop_processor_route)
            .json(&json!(request_body))
            .await;
        stop_processor_response.assert_status_ok();
        let response_details = stop_processor_response.json::<ResponseDetails>();
        assert_eq!(response_details.status, super::ProcessorStatus::Stopped);
        let get_status_response = test_server
            .get(get_status_route)
            .json(&json!(request_body))
            .await;
        get_status_response.assert_status_ok();
        let response_details = get_status_response.json::<ResponseDetails>();
        assert_eq!(
            response_details.status,
            super::ProcessorStatus::Stopped,
            "Processor status should be 'Stopped' after we stop the processor."
        );

        cancellation_token.cancel();
    }

    #[tokio::test]
    async fn test_stop_processor() {
        let create_processor_route = "/create_processor";
        let start_processor_route = "/start_processor";
        let stop_processor_route = "/stop_processor";

        let config: MainConfig = MainConfig {
            server_port: 8080,
            processor_queue_length: 10,
        };

        let processor_mappings = HashMap::from([
            (
                "adder_processor".to_string(),
                ProcessorType::SourceProcessor,
            ),
            ("doubler_processor".to_string(), ProcessorType::Other),
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
            .route(&create_processor_route, post(super::create_processor))
            .route(&start_processor_route, patch(super::start_processor))
            .route(&stop_processor_route, patch(super::stop_processor))
            .with_state(state);

        let test_server = TestServer::new(app).unwrap();
        let request_body: RequestDetails = RequestDetails {
            processor_name: "adder_processor".to_string(),
            processor_id: None,
        };

        let response = test_server
            .post(create_processor_route)
            .json(&json!(request_body))
            .await;
        response.assert_status_ok();

        let response_details = response.json::<ResponseDetails>();
        assert_eq!(response_details.status, super::ProcessorStatus::Stopped);

        let request_body: RequestDetails = RequestDetails {
            processor_name: "adder_processor".to_string(),
            processor_id: Some(response_details.processor_id),
        };

        let response = test_server
            .patch(start_processor_route)
            .json(&json!(request_body))
            .await;
        response.assert_status_ok();
        let response_details = response.json::<ResponseDetails>();
        assert_eq!(response_details.status, super::ProcessorStatus::Running);

        let response = test_server
            .patch(stop_processor_route)
            .json(&json!(request_body))
            .await;
        response.assert_status_ok();
        let response_details = response.json::<ResponseDetails>();
        assert_eq!(response_details.status, super::ProcessorStatus::Stopped);
        cancellation_token.cancel();
    }

    #[tokio::test]
    async fn test_start_processor() {
        let create_processor_route = "/create_processor";
        let start_processor_route = "/start_processor";

        let config: MainConfig = MainConfig {
            server_port: 8080,
            processor_queue_length: 10,
        };

        let processor_mappings = HashMap::from([
            (
                "adder_processor".to_string(),
                ProcessorType::SourceProcessor,
            ),
            ("doubler_processor".to_string(), ProcessorType::Other),
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
            .route(&create_processor_route, post(super::create_processor))
            .route(&start_processor_route, patch(super::start_processor))
            .with_state(state);

        let test_server = TestServer::new(app).unwrap();
        let request_body: RequestDetails = RequestDetails {
            processor_name: "adder_processor".to_string(),
            processor_id: None,
        };

        let response = test_server
            .post(create_processor_route)
            .json(&json!(request_body))
            .await;
        response.assert_status_ok();

        let response_details = response.json::<ResponseDetails>();
        assert_eq!(response_details.status, super::ProcessorStatus::Stopped);

        let request_body: RequestDetails = RequestDetails {
            processor_name: "adder_processor".to_string(),
            processor_id: Some(response_details.processor_id),
        };

        let response = test_server
            .patch(start_processor_route)
            .json(&json!(request_body))
            .await;
        response.assert_status_ok();
        let response_details = response.json::<ResponseDetails>();
        assert_eq!(response_details.status, super::ProcessorStatus::Running);
        cancellation_token.cancel();
    }
}
