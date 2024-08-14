use crate::processors::models::{Message, ProcessorStatus};

use super::{
    base_processor::SourceProcessor,
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
    peers_tx: Vec<mpsc::Sender<Message>>,
    cancellation_token: CancellationToken,
}

impl SourceProcessor for InMemorySourceProcessor {
    fn new(
        processor_name: String,
        parent_rx: mpsc::Receiver<ProcessorCommand>,
        peer_processors_tx: Vec<mpsc::Sender<Message>>,
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
                    }
                }
                _ = self.cancellation_token.cancelled() => {
                    tracing::info!("{}: Cancellation token received. Shutting down.", self.processor_name);
                    break;
                }
                else => {
                    if self.status == ProcessorStatus::Running {
                        if let Some(packet) = generate_packet_func() {
                            for tx in self.peers_tx.iter() {
                                tx.send(Message::InMemoryMessage(packet.clone())).await.unwrap();
                            }
                        }
                    }
                }
            }
        }
    }
}
