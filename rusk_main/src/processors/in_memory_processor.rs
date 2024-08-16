use crate::{handlers::models::ProcessorInfo, processors::models::ProcessorStatus};

use super::{
    base_processor::{ProcessorConnection, SinkProcessor},
    models::{InMemoryPacket, Message, ProcessorCommand},
};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct InMemoryProcessor {
    pub processor_name: String,
    pub processor_id: Uuid,
    status: super::models::ProcessorStatus,
    parent_rx: mpsc::Receiver<ProcessorCommand>,
    peers_rx: mpsc::Receiver<Message>,
    peers_tx: HashMap<Uuid, mpsc::Sender<Message>>,
    cancellation_token: CancellationToken,
    packets_processed_count: u64,
}

impl SinkProcessor for InMemoryProcessor {
    fn new(
        processor_name: String,
        peers_rx: mpsc::Receiver<Message>,
        parent_rx: mpsc::Receiver<ProcessorCommand>,
        cancellation_token: CancellationToken,
    ) -> Self {
        InMemoryProcessor {
            processor_name,
            processor_id: Uuid::new_v4(),
            status: super::models::ProcessorStatus::Stopped,
            parent_rx,
            peers_rx,
            peers_tx: HashMap::new(),
            cancellation_token,
            packets_processed_count: 0,
        }
    }
}

impl ProcessorConnection for InMemoryProcessor {
    fn connect_processor(&mut self, receiver_processor_id: Uuid, tx: mpsc::Sender<Message>) {
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
                        ProcessorCommand::Stop {resp} => {
                            self.status = ProcessorStatus::Stopped;
                            println!("{}: Stopped", self.processor_name);
                            resp.send(self.status).unwrap();
                        }
                        ProcessorCommand::Start {resp} => {
                            self.status = ProcessorStatus::Running;
                            println!("{}: Started running...", self.processor_name);
                            resp.send(self.status).unwrap();
                        }
                        ProcessorCommand::GetStatus {resp} => {
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
                                packets_processed_count: self.packets_processed_count,
                            };
                            resp.send(processor_info).unwrap();
                        }
                    }
                }
                Some(command) = self.peers_rx.recv() => {
                    println!("Received command from a peer");
                    match command {
                        Message::InMemoryMessage(packet) if self.status == ProcessorStatus::Running => {
                            tracing::info!(
                                "{}: Received packet from someone. Processing it.",
                                self.processor_name
                            );
                            println!(
                                "{}: Received packet from someone. Processing it.",
                                self.processor_name
                            );

                            if !self.peers_tx.is_empty() {
                            let processed_packet = process_packet_func(packet);
                            match processed_packet {
                                Some(packet) => {
                                    for tx in self.peers_tx.values() {
                                        let _ = tx
                                            .send(Message::InMemoryMessage(packet.clone()))
                                            .await;
                                    }
                                    tracing::info!("{}: Sent packet to {} processors", self.processor_name, self.peers_tx.len());
                                    self.packets_processed_count += 1;
                                    tracing::info!("{}: Processed {} packets. Will sleep now for a while.", self.processor_name, self.packets_processed_count);
                                }
                                None => {
                                    tracing::error!(
                                        "{}: Error processing packet", self.processor_name);
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

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use super::*;
    use crate::{doubler_func, processors::models::ProcessorStatus};
    use tokio::sync::{mpsc, oneshot};

    #[tokio::test]
    async fn test_in_memory_processor() {
        let (parent_tx, parent_rx) = mpsc::channel(10);
        let (peers_tx, peers_rx) = mpsc::channel(10);
        let cancellation_token = CancellationToken::new();

        let mut sink_processor = InMemoryProcessor::new(
            "test_in_memory_processor".to_string(),
            peers_rx,
            parent_rx,
            cancellation_token.clone(),
        );

        tokio::spawn(async move {
            sink_processor.run(doubler_func).await;
        });

        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        let command = ProcessorCommand::Start { resp: oneshot_tx };

        parent_tx.send(command).await.unwrap();
        let status = oneshot_rx.await.unwrap();
        assert_eq!(status, ProcessorStatus::Running);

        sleep(Duration::from_millis(1000));

        let message = Message::InMemoryMessage(InMemoryPacket {
            id: Uuid::new_v4(),
            data: vec![1, 2, 3, 4],
        });

        peers_tx.send(message).await.unwrap();

        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        let command = ProcessorCommand::GetInfo { resp: oneshot_tx };

        parent_tx.send(command).await.unwrap();
        let status = oneshot_rx.await.unwrap();
        println!("{:?}", status);

        cancellation_token.cancel();
    }
}
