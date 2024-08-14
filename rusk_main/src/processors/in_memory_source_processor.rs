use crate::processors::models::ProcessorStatus;

use super::{
    base_processor::ProcessorV2,
    models::{InMemoryPacket, ProcessorCommand},
};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct InMemorySourceProcessor {
    pub processor_name: String,
    pub processor_id: Uuid,
    pub status: super::models::ProcessorStatus,
    parent_tx: mpsc::Sender<ProcessorCommand>,
    parent_rx: mpsc::Receiver<ProcessorCommand>,
    peers_tx: HashMap<Uuid, mpsc::Sender<ProcessorCommand>>,
    cancellation_token: CancellationToken,
}

impl ProcessorV2 for InMemorySourceProcessor {
    fn new(
        processor_name: String,
        parent_tx: mpsc::Sender<ProcessorCommand>,
        parent_rx: mpsc::Receiver<ProcessorCommand>,
        cancellation_token: CancellationToken,
        peers_rx: mpsc::Receiver<ProcessorCommand>,
    ) -> Self {
        InMemorySourceProcessor {
            processor_name,
            processor_id: Uuid::new_v4(),
            status: super::models::ProcessorStatus::Stopped,
            parent_tx,
            parent_rx,
            peers_tx: HashMap::new(),
            cancellation_token,
        }
    }

    fn connect_processor(
        &mut self,
        receiver_processor_id: Uuid,
        tx: mpsc::Sender<ProcessorCommand>,
    ) {
        self.peers_tx.insert(receiver_processor_id, tx);
    }

    fn disconnect_processor(&mut self, receiver_processor_id: Uuid) {
        self.peers_tx.remove(&receiver_processor_id);
    }
}

impl InMemorySourceProcessor {
    pub async fn run(&mut self, generate_packet_func: fn() -> Option<InMemoryPacket>) {
        loop {
            match self.status {
                ProcessorStatus::Running => {
                    if let Some(packet) = generate_packet_func() {
                        for tx in self.peers_tx.values() {
                            tx.send(ProcessorCommand::InMemoryMessage(packet.clone()))
                                .await
                                .unwrap();
                        }
                    }
                    tokio::select! {
                        Some(command) = self.parent_rx.recv() => {
                            match command {
                                ProcessorCommand::Stop => {
                                    self.status = ProcessorStatus::Stopped;
                                    self.parent_tx.send(ProcessorCommand::Result(self.status)).await.unwrap();
                                }
                                _ => {}
                            }
                        }
                        _ = self.cancellation_token.cancelled() => {
                            tracing::info!("Cancellation token received. Shutting down processor {}", self.processor_name);
                            break;
                        }
                    }
                }
                ProcessorStatus::Stopped => {
                    tokio::select! {
                        Some(command) = self.parent_rx.recv() => {
                            match command {
                                ProcessorCommand::Start => {
                                    self.status = ProcessorStatus::Running;
                                    self.parent_tx.send(ProcessorCommand::Result(self.status)).await.unwrap();
                                }
                                _ => {}
                            }
                        }
                        _ = self.cancellation_token.cancelled() => {
                            tracing::info!("Cancellation token received. Shutting down processor {}", self.processor_name);
                            break;
                        }
                    }
                }
                ProcessorStatus::Errored => {
                    break;
                }
            }
        }
    }
}
