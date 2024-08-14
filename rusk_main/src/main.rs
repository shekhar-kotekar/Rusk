use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::FromRef,
    routing::{delete, get, post},
    Router,
};
use commons::MainConfig;
use http::{header, Method};
use processors::models::{InMemoryPacket, Message, ProcessorCommand};
use rand::Rng;
use tokio::{
    signal,
    sync::{mpsc, Mutex},
};
use tokio_util::sync::CancellationToken;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

mod handlers;
mod processors;

#[derive(Clone, Debug, FromRef)]
struct AppState {
    config: MainConfig,
    cancellation_token: CancellationToken,
    peers_tx: Arc<Mutex<HashMap<Uuid, mpsc::Sender<Message>>>>,
    parent_processor_tx: Arc<Mutex<HashMap<Uuid, mpsc::Sender<ProcessorCommand>>>>,
}

#[tokio::main]
async fn main() {
    commons::enable_tracing();
    //console_subscriber::init();

    let main_config: MainConfig = commons::get_config().rusk_main;
    let cancellation_token = CancellationToken::new();
    let state = AppState {
        config: main_config.clone(),
        cancellation_token: cancellation_token.clone(),
        peers_tx: Arc::new(Mutex::new(HashMap::new())),
        parent_processor_tx: Arc::new(Mutex::new(HashMap::new())),
    };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE])
        .allow_origin(Any);

    // TODO: Add route for pause and resume processor
    let server = Router::new()
        .route("/is_alive", get(handlers::is_alive))
        .route("/processor/delete", delete(handlers::delete_processor))
        .route("/processor/stop", post(handlers::stop_processor))
        .route("/processor/start", post(handlers::start_processor))
        .route("/processor/create", post(handlers::create_processor))
        .route("/processor/get_status", get(handlers::get_status))
        .route("/processor/get_info", get(handlers::get_processor_info))
        .route("/processor/connect", post(handlers::connect_processors))
        .route(
            "/processor/disconnect",
            post(handlers::disconnect_processors),
        )
        .layer(cors)
        .with_state(state);

    let server_address = format!("0.0.0.0:{}", main_config.server_port);

    tracing::info!("Starting rusk server on {}", server_address);

    let listener = tokio::net::TcpListener::bind(server_address).await.unwrap();

    axum::serve(listener, server)
        .with_graceful_shutdown(shutdown_signal(cancellation_token))
        .await
        .unwrap();

    // let (tx_for_doubler, rx_for_doubler) =
    //     mpsc::channel::<ProcessorCommand>(main_config.processor_queue_length);

    // let mut doubler_processor = InMemoryProcessor::new("Doubler".to_string(), rx_for_doubler);
    // adder_processor.add_tx(tx_for_doubler.clone());
    // processor_tx.insert(doubler_processor.processor_id, tx_for_doubler);

    // let adder_handle = tokio::spawn(async move {
    //     adder_processor.run(adder_func).await;
    // });
    // let doubler_handle = tokio::spawn(async move {
    //     doubler_processor.run(doubler_func).await;
    // });

    // for (_, tx) in processor_tx.iter() {
    //     tx.send(ProcessorCommand::Start).await.unwrap();
    // }
}

async fn shutdown_signal(cancellation_token: CancellationToken) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl-C signal, shutting down...");
            cancellation_token.cancel();
        },
        _ = terminate => {
            tracing::info!("Received terminate signal, shutting down...");
            cancellation_token.cancel();
        },
    }
}

fn adder_func() -> Option<InMemoryPacket> {
    let mut rng = rand::thread_rng();
    let data: Vec<u8> = (0..3).map(|_| rng.gen_range(1..100)).collect();
    Some(InMemoryPacket {
        id: Uuid::new_v4(),
        data,
    })
}

// fn doubler_func(packet: InMemoryPacket) -> Option<InMemoryPacket> {
//     let new_data = packet.data.iter().map(|x| x * 2).collect();
//     let new_packet = InMemoryPacket {
//         id: packet.id,
//         data: new_data,
//     };
//     tracing::info!(
//         "old data: {:?}, new data: {:?}",
//         packet.data,
//         new_packet.data
//     );
//     Some(new_packet)
// }
