use std::collections::HashMap;

use crate::{
    adder_func,
    processors::{
        base_processor::SourceProcessor,
        in_memory_source_processor::InMemorySourceProcessor,
        models::{ProcessorCommand, ProcessorStatus},
    },
    AppState,
};
use axum::{debug_handler, extract::State, Json};
use http::StatusCode;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use super::models::{ProcessorInfo, RequestDetails, ResponseDetails};

const PARENT_PROCESSOR_CHANNEL_SIZE: usize = 10;

#[tracing::instrument]
pub async fn create_processor(
    State(server_state): State<AppState>,
    Json(payload): Json<RequestDetails>,
) -> Result<Json<ResponseDetails>, StatusCode> {
    let (parent_to_processor_tx, processor_to_parent_rx) =
        mpsc::channel::<ProcessorCommand>(PARENT_PROCESSOR_CHANNEL_SIZE);

    let mut processor = InMemorySourceProcessor::new(
        payload.processor_name,
        processor_to_parent_rx,
        HashMap::new(),
        server_state.cancellation_token.clone(),
    );

    let processor_status = processor.status;
    let processor_id = processor.processor_id;

    tokio::spawn(async move {
        processor.run(adder_func).await;
    });

    server_state
        .parent_processor_tx
        .lock()
        .await
        .insert(processor_id, parent_to_processor_tx);

    tracing::info!("Processor created: {}", processor_id);
    let result = Json(ResponseDetails {
        processor_id: processor_id.to_string(),
        status: processor_status,
    });
    return Ok(result);
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
) -> Result<Json<ProcessorInfo>, StatusCode> {
    let processor_info = ProcessorInfo {
        processor_id: "DUMMY PROCESSOR ID".to_string(),
        status: ProcessorStatus::Stopped,
        number_of_packets_processed: 0,
    };
    Ok(Json(processor_info))
}

#[tracing::instrument]
pub async fn connect_processors(State(server_state): State<AppState>) -> &'static str {
    "NOT IMPLEMENTED YET!"
}

#[tracing::instrument]
pub async fn disconnect_processors(State(server_state): State<AppState>) -> &'static str {
    "NOT IMPLEMENTED YET!"
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use axum::{
        routing::{get, post},
        Router,
    };
    use axum_test::TestServer;
    use commons::MainConfig;
    use serde_json::json;
    use tokio::sync::Mutex;
    use tokio_util::sync::CancellationToken;

    use crate::handlers::models::{RequestDetails, ResponseDetails};

    #[tokio::test]
    async fn test_create_processor() {
        let route = "/create_processor";

        let config: MainConfig = MainConfig {
            server_port: 8080,
            processor_queue_length: 10,
        };
        let cancellation_token = CancellationToken::new();
        let state = super::AppState {
            config,
            cancellation_token: cancellation_token.clone(),
            peers_tx: Arc::new(Mutex::new(HashMap::new())),
            parent_processor_tx: Arc::new(Mutex::new(HashMap::new())),
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

        let cancellation_token = CancellationToken::new();
        let state = super::AppState {
            config,
            cancellation_token: cancellation_token.clone(),
            peers_tx: Arc::new(Mutex::new(HashMap::new())),
            parent_processor_tx: Arc::new(Mutex::new(HashMap::new())),
        };

        let app = Router::new()
            .route(&create_processor_route, post(super::create_processor))
            .route(&start_processor_route, post(super::start_processor))
            .route(&stop_processor_route, post(super::stop_processor))
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
            .post(start_processor_route)
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
            .post(stop_processor_route)
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

        let cancellation_token = CancellationToken::new();
        let state = super::AppState {
            config,
            cancellation_token: cancellation_token.clone(),
            peers_tx: Arc::new(Mutex::new(HashMap::new())),
            parent_processor_tx: Arc::new(Mutex::new(HashMap::new())),
        };

        let app = Router::new()
            .route(&create_processor_route, post(super::create_processor))
            .route(&start_processor_route, post(super::start_processor))
            .route(&stop_processor_route, post(super::stop_processor))
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
            .post(start_processor_route)
            .json(&json!(request_body))
            .await;
        response.assert_status_ok();
        let response_details = response.json::<ResponseDetails>();
        assert_eq!(response_details.status, super::ProcessorStatus::Running);

        let response = test_server
            .post(stop_processor_route)
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
        let cancellation_token = CancellationToken::new();
        let state = super::AppState {
            config,
            cancellation_token: cancellation_token.clone(),
            peers_tx: Arc::new(Mutex::new(HashMap::new())),
            parent_processor_tx: Arc::new(Mutex::new(HashMap::new())),
        };

        let app = Router::new()
            .route(&create_processor_route, post(super::create_processor))
            .route(&start_processor_route, post(super::start_processor))
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
            .post(start_processor_route)
            .json(&json!(request_body))
            .await;
        response.assert_status_ok();
        let response_details = response.json::<ResponseDetails>();
        assert_eq!(response_details.status, super::ProcessorStatus::Running);
        cancellation_token.cancel();
    }
}
