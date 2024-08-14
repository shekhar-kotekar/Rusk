use crate::processors::models::ProcessorStatus;

use super::{
    base_processor::ProcessorV2,
    models::{InMemoryPacket, ProcessorCommand},
};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct InMemoryProcessor {
    pub processor_name: String,
    pub processor_id: Uuid,
    pub status: super::models::ProcessorStatus,
    parent_tx: mpsc::Sender<ProcessorCommand>,
    parent_rx: mpsc::Receiver<ProcessorCommand>,
    peers_rx: mpsc::Receiver<ProcessorCommand>,
    peers_tx: HashMap<Uuid, mpsc::Sender<ProcessorCommand>>,
    cancellation_token: CancellationToken,
}

impl ProcessorV2 for InMemoryProcessor {
    fn new(
        processor_name: String,
        parent_tx: mpsc::Sender<ProcessorCommand>,
        parent_rx: mpsc::Receiver<ProcessorCommand>,
        cancellation_token: CancellationToken,
        peers_rx: mpsc::Receiver<ProcessorCommand>,
    ) -> Self {
        InMemoryProcessor {
            processor_name,
            processor_id: Uuid::new_v4(),
            status: super::models::ProcessorStatus::Stopped,
            parent_tx,
            parent_rx,
            peers_rx: peers_rx,
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

impl InMemoryProcessor {
    pub async fn run(&mut self, process_packet_func: fn(InMemoryPacket) -> Option<InMemoryPacket>) {
        loop {
            tokio::select! {
                Some(command) = self.parent_rx.recv() => {
                    match command {
                        ProcessorCommand::Stop => {
                            self.status = ProcessorStatus::Stopped;
                        }
                        ProcessorCommand::Start => {
                            self.status = ProcessorStatus::Running;
                        }
                        _ => {}
                    }
                    let _ = self.parent_tx.send(ProcessorCommand::Result(self.status)).await;
                }
                Some(command) = self.peers_rx.recv() => {
                    match command {
                        ProcessorCommand::InMemoryMessage(packet) if self.status == ProcessorStatus::Running => {
                            tracing::info!(
                                "{}: Received packet from someone. Processing it.",
                                self.processor_name
                            );
                            if self.peers_tx.len() > 0 {
                            let processed_packet = process_packet_func(packet);
                            match processed_packet {
                                Some(packet) => {
                                    for tx in self.peers_tx.values() {
                                        let _ = tx
                                            .send(ProcessorCommand::InMemoryMessage(packet.clone()))
                                            .await;
                                    }
                                }
                                None => {
                                    // write code to send error to error channel
                                }
                            }
                        }
                        }
                        _ => {}
                    }
                }
                _ = self.cancellation_token.cancelled() => {
                    tracing::info!("{}: Cancellation token received. Shutting down.", self.processor_name);
                    break;
                }
            }
        }
    }
}
