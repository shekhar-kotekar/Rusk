use std::collections::HashMap;

use crate::{
    handlers::models::ProcessorInfo,
    processors::models::{Message, ProcessorStatus},
};

use super::{
    base_processor::{ProcessorConnection, SourceProcessor},
    models::{InMemoryPacket, ProcessorCommand},
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct InMemorySourceProcessor {
    pub processor_name: String,
    pub processor_id: Uuid,
    pub status: super::models::ProcessorStatus,
    parent_rx: mpsc::Receiver<ProcessorCommand>,
    peers_tx: HashMap<Uuid, mpsc::Sender<Message>>,
    cancellation_token: CancellationToken,
}

impl SourceProcessor for InMemorySourceProcessor {
    fn new(
        processor_name: String,
        parent_rx: mpsc::Receiver<ProcessorCommand>,
        peer_processors_tx: HashMap<Uuid, mpsc::Sender<Message>>,
        cancellation_token: CancellationToken,
    ) -> Self {
        InMemorySourceProcessor {
            processor_name,
            processor_id: Uuid::new_v4(),
            status: super::models::ProcessorStatus::Stopped,
            parent_rx,
            peers_tx: peer_processors_tx,
            cancellation_token,
        }
    }
}

impl ProcessorConnection for InMemorySourceProcessor {
    fn disconnect_processor(&mut self, receiver_processor_id: Uuid) {
        self.peers_tx.remove(&receiver_processor_id);
    }

    fn connect_processor(
        &mut self,
        receiver_processor_id: Uuid,
        tx_for_receiver: mpsc::Sender<Message>,
    ) {
        self.peers_tx.insert(receiver_processor_id, tx_for_receiver);
    }
}

impl InMemorySourceProcessor {
    pub async fn run(&mut self, generate_packet_func: fn() -> Option<InMemoryPacket>) {
        loop {
            tokio::select! {
                Some(command) = self.parent_rx.recv() => {
                    match command {
                        ProcessorCommand::Start {resp} => {
                            self.status = ProcessorStatus::Running;
                            resp.send(self.status).unwrap();
                        }
                        ProcessorCommand::Stop {resp} => {
                            self.status = ProcessorStatus::Stopped;
                            resp.send(self.status).unwrap();
                        }
                        ProcessorCommand::GetStatus {resp} => {
                            println!("status of {}: {:?}", self.processor_name, self.status);
                            resp.send(self.status).unwrap();
                        }
                        ProcessorCommand::Connect {destination_processor_id, destination_processor_tx, resp} => {
                            self.connect_processor(destination_processor_id, destination_processor_tx);
                            resp.send(self.status).unwrap();
                        }
                        ProcessorCommand::Disconnect {destination_processor_id, resp} => {
                            self.disconnect_processor(destination_processor_id);
                            resp.send(self.status).unwrap();
                        }
                        ProcessorCommand::GetInfo {resp} => {
                            let processor_info = ProcessorInfo {
                                processor_id: self.processor_id.to_string(),
                                status: self.status,
                                number_of_packets_processed: 0,
                            };
                            resp.send(processor_info).unwrap();
                        }
                    }
                }
                _ = self.cancellation_token.cancelled() => {
                    tracing::info!("{}: Cancellation token received. Shutting down.", self.processor_name);
                    break;
                }
                else => {
                    if self.status == ProcessorStatus::Running {
                        if let Some(packet) = generate_packet_func() {
                            for tx in self.peers_tx.values() {
                                tx.send(Message::InMemoryMessage(packet.clone())).await.unwrap();
                            }
                        }
                    }
                }
            }
        }
    }
}
