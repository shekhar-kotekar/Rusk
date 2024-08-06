use tokio::sync::mpsc;
use uuid::Uuid;

use crate::processors::models::ProcessorCommand;

use super::{
    base_processor::Processor,
    models::{InMemoryPacket, ProcessorStatus},
};

pub struct InMemoryProcessor {
    pub processor_name: String,
    pub processor_id: Uuid,
    tx: Vec<mpsc::Sender<ProcessorCommand>>,
    rx: mpsc::Receiver<ProcessorCommand>,
    pub status: ProcessorStatus,
}

impl Processor for InMemoryProcessor {
    fn new(processor_name: String, rx: mpsc::Receiver<ProcessorCommand>) -> InMemoryProcessor {
        InMemoryProcessor {
            processor_name,
            processor_id: Uuid::new_v4(),
            tx: Vec::new(),
            rx,
            status: ProcessorStatus::Stopped,
        }
    }
    fn add_tx(&mut self, tx: mpsc::Sender<ProcessorCommand>) {
        self.tx.push(tx);
    }
}

impl InMemoryProcessor {
    pub async fn run(&mut self, process_packet_func: fn(InMemoryPacket) -> Option<InMemoryPacket>) {
        loop {
            match self.status {
                ProcessorStatus::Running => {
                    let result = self.rx.recv().await;
                    match result {
                        Some(ProcessorCommand::InMemoryMessage(packet)) => {
                            tracing::info!(
                                "{}: Processing packet : {:?}",
                                self.processor_name,
                                packet
                            );
                            let processed_packet = process_packet_func(packet);
                            match processed_packet {
                                Some(packet) => {
                                    for tx in self.tx.iter() {
                                        let new_command =
                                            ProcessorCommand::InMemoryMessage(packet.clone());
                                        tx.send(new_command).await.unwrap();
                                    }
                                }
                                None => {
                                    // write code to send error to error channel
                                }
                            }
                        }
                        Some(ProcessorCommand::Stop) => {
                            tracing::info!("{}: Shutting down", self.processor_name);
                            break;
                        }
                        Some(ProcessorCommand::Pause) => {
                            tracing::info!("{}: Pausing", self.processor_name);
                            self.status = ProcessorStatus::Paused;
                        }
                        _ => {
                            // write code to send error to error channel
                        }
                    }
                }
                ProcessorStatus::Paused => {
                    let result = self.rx.recv().await;
                    match result {
                        Some(ProcessorCommand::Resume) => {
                            self.status = ProcessorStatus::Running;
                            tracing::info!("{}: Resumed", self.processor_name);
                        }
                        Some(ProcessorCommand::Stop) => {
                            tracing::info!("{}: Shutting down", self.processor_name);
                            break;
                        }
                        _ => {}
                    }
                }
                ProcessorStatus::Stopped => {
                    tracing::info!(
                        "{} processor is Stopped. Waiting for command on rx channel.",
                        self.processor_name
                    );
                    let result = self.rx.recv().await;
                    match result {
                        Some(ProcessorCommand::Start) => {
                            self.status = ProcessorStatus::Running;
                            tracing::info!("{}: Started", self.processor_name);
                        }
                        Some(ProcessorCommand::Stop) => {
                            tracing::info!("{}: Shutting down", self.processor_name);
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
