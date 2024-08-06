use crate::{
    adder_func,
    processors::{
        base_processor::Processor,
        in_memory_source_processor::InMemorySourceProcessor,
        models::{ProcessorCommand, ProcessorStatus},
    },
    AppState,
};
use axum::{debug_handler, extract::State, Json};
use http::StatusCode;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RequestDetails {
    name: String,
    processor_id: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ResponseDetails {
    processor_id: String,
    status: ProcessorStatus,
}

#[tracing::instrument]
pub async fn is_alive() -> &'static str {
    "I am alive!"
}

#[tracing::instrument]
pub async fn create_processor(
    State(server_state): State<AppState>,
    Json(payload): Json<RequestDetails>,
) -> Result<Json<ResponseDetails>, StatusCode> {
    let (tx_for_adder, rx_for_adder) =
        mpsc::channel::<ProcessorCommand>(server_state.config.processor_queue_length);

    let mut processor = InMemorySourceProcessor::new(payload.name, rx_for_adder);
    let processor_status = processor.status;
    let processor_id = processor.processor_id;

    tokio::spawn(async move {
        processor.run(adder_func).await;
    });

    server_state
        .processor_senders
        .lock()
        .await
        .insert(processor_id, tx_for_adder);

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
        .processor_senders
        .lock()
        .await
        .get(&processor_id)
    {
        Some(tx) => {
            tracing::info!("Starting processor: {}", processor_id);
            let _ = tx.send(ProcessorCommand::Start).await;
            let result = Json(ResponseDetails {
                processor_id: processor_id.to_string(),
                status: ProcessorStatus::Running,
            });
            return Ok(result);
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
        .processor_senders
        .lock()
        .await
        .get(&processor_id)
    {
        Some(tx) => {
            let _ = tx.send(ProcessorCommand::Stop).await;
            let result = Json(ResponseDetails {
                processor_id: processor_id.to_string(),
                status: ProcessorStatus::Stopped,
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
pub async fn delete_processor(State(server_state): State<AppState>) -> &'static str {
    "NOT IMPLEMENTED YET!"
}

#[tracing::instrument]
pub async fn get_processor_status(
    State(server_state): State<AppState>,
    Json(payload): Json<RequestDetails>,
) -> Result<Json<ResponseDetails>, StatusCode> {
    // TODO: How to get processor status?
    let processor_id = Uuid::parse_str(&payload.processor_id.unwrap()).unwrap();
    match server_state
        .processor_senders
        .lock()
        .await
        .get(&processor_id)
    {
        Some(processor_tx) => {
            let _ = processor_tx.send(ProcessorCommand::Stop).await;
            let result = Json(ResponseDetails {
                processor_id: processor_id.to_string(),
                status: ProcessorStatus::Running,
            });
            return Ok(result);
        }
        None => {
            return Err(StatusCode::NOT_FOUND);
        }
    }
}

#[tracing::instrument]
pub async fn get_processor_info(State(server_state): State<AppState>) -> &'static str {
    "get processor info"
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

    use crate::handlers::{RequestDetails, ResponseDetails};

    #[tokio::test]
    async fn test_is_alive() {
        let app = Router::new().route("/is_alive", get(super::is_alive));
        let test_server = TestServer::new(app).unwrap();
        let response = test_server.get("/is_alive").await;
        response.assert_status_ok();
        response.assert_text("I am alive!");
    }

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
            processor_senders: Arc::new(Mutex::new(HashMap::new())),
        };

        let app = Router::new()
            .route(&route, post(super::create_processor))
            .with_state(state);

        let test_server = TestServer::new(app).unwrap();
        let request_body: RequestDetails = RequestDetails {
            name: "adder_processor".to_string(),
            processor_id: None,
        };

        let response = test_server.post(route).json(&json!(request_body)).await;
        response.assert_status_ok();

        let response_details = response.json::<ResponseDetails>();
        assert_eq!(response_details.status, super::ProcessorStatus::Stopped);
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
            processor_senders: Arc::new(Mutex::new(HashMap::new())),
        };

        let app = Router::new()
            .route(&create_processor_route, post(super::create_processor))
            .route(&start_processor_route, post(super::start_processor))
            .route(&stop_processor_route, post(super::stop_processor))
            .with_state(state);

        let test_server = TestServer::new(app).unwrap();
        let request_body: RequestDetails = RequestDetails {
            name: "adder_processor".to_string(),
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
            name: "adder_processor".to_string(),
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
            processor_senders: Arc::new(Mutex::new(HashMap::new())),
        };

        let app = Router::new()
            .route(&create_processor_route, post(super::create_processor))
            .route(&start_processor_route, post(super::start_processor))
            .with_state(state);

        let test_server = TestServer::new(app).unwrap();
        let request_body: RequestDetails = RequestDetails {
            name: "adder_processor".to_string(),
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
            name: "adder_processor".to_string(),
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
