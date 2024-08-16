use std::{collections::HashMap, time::Duration};

use crate::{
    handlers::models::ProcessorInfo,
    processors::models::{Message, ProcessorStatus},
};

use super::{
    base_processor::{ProcessorConnection, SourceProcessor},
    models::{InMemoryPacket, ProcessorCommand},
};
use tokio::{sync::mpsc, time::sleep};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct InMemorySourceProcessor {
    pub processor_name: String,
    pub processor_id: Uuid,
    pub status: super::models::ProcessorStatus,
    parent_rx: mpsc::Receiver<ProcessorCommand>,
    peers_tx: HashMap<Uuid, mpsc::Sender<Message>>,
    cancellation_token: CancellationToken,
    packets_processed_count: u64,
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
            packets_processed_count: 0,
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
                            println!("{}: Started running...", self.processor_name);
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
                                packets_processed_count: self.packets_processed_count,
                            };
                            resp.send(processor_info).unwrap();
                        }
                    }
                }
                _ = self.cancellation_token.cancelled() => {
                    tracing::info!("{}: Cancellation token received. Shutting down.", self.processor_name);
                    println!("{}: Cancellation token received. Shutting down.", self.processor_name);
                    break;
                }
                _ = sleep(Duration::from_millis(100)) => {
                    if self.status == ProcessorStatus::Running && !self.peers_tx.is_empty() {
                            if let Some(packet) = generate_packet_func() {
                                for tx in self.peers_tx.values() {
                                    tx.send(Message::InMemoryMessage(packet.clone())).await.unwrap();
                                }
                                tracing::info!("{}: Sent packet to {} processors", self.processor_name, self.peers_tx.len());
                                self.packets_processed_count += 1;
                            tracing::info!("{}: Processed {} packets. Will sleep now for a while.", self.processor_name, self.packets_processed_count);
                            }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adder_func;
    use tokio::sync::{mpsc, oneshot};

    #[tokio::test]
    async fn test_in_memory_source_processor() {
        let (parent_tx, parent_rx) = mpsc::channel(5);
        let cancellation_token = CancellationToken::new();
        let mut processor = InMemorySourceProcessor::new(
            "test_processor".to_string(),
            parent_rx,
            HashMap::new(),
            cancellation_token.clone(),
        );
        tokio::spawn(async move {
            processor.run(adder_func).await;
        });
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        let command = ProcessorCommand::Start { resp: oneshot_tx };

        parent_tx.send(command).await.unwrap();
        let status = oneshot_rx.await.unwrap();
        assert_eq!(status, ProcessorStatus::Running);

        sleep(Duration::from_secs(1)).await;

        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        let command = ProcessorCommand::GetInfo { resp: oneshot_tx };
        parent_tx.send(command).await.unwrap();
        let processor_info = oneshot_rx.await.unwrap();
        println!("{:?}", processor_info);
        //assert_eq!(processor_info.packets_processed_count, 1);

        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        let command = ProcessorCommand::GetInfo { resp: oneshot_tx };
        sleep(Duration::from_secs(2)).await;
        parent_tx.send(command).await.unwrap();
        let processor_info = oneshot_rx.await.unwrap();
        println!("{:?}", processor_info);

        cancellation_token.cancel();
    }
}
