use tokio::sync::broadcast;
use uuid::Uuid;

use crate::processors::models::ProcessorCommand;

use super::models::InMemoryPacket;

pub struct InMemoryProcessor {
    pub processor_name: String,
    pub processor_id: Uuid,
    pub tx: broadcast::Sender<ProcessorCommand>,
    pub rx: broadcast::Receiver<ProcessorCommand>,
    paused: bool,
}

impl InMemoryProcessor {
    pub fn new(
        processor_name: String,
        tx: broadcast::Sender<ProcessorCommand>,
        rx: broadcast::Receiver<ProcessorCommand>,
    ) -> InMemoryProcessor {
        InMemoryProcessor {
            processor_name,
            processor_id: Uuid::new_v4(),
            tx,
            rx,
            paused: false,
        }
    }

    pub async fn run(&mut self, process_packet_func: fn(InMemoryPacket) -> InMemoryPacket) {
        loop {
            if self.paused {
                tokio::select! {
                    command = self.rx.recv() => {
                        match command {
                            Ok(ProcessorCommand::Resume) => {
                                self.paused = false;
                                tracing::info!("{}: Resumed", self.processor_name);
                            },
                            Ok(ProcessorCommand::Shutdown) => {
                                tracing::info!("{}: Shutting down", self.processor_name);
                                break;
                            },
                            _ => {}
                        }
                    }
                }
            } else {
                tracing::info!("{}: is paused? {}", self.processor_name, self.paused);
                tokio::select! {
                    command = self.rx.recv() => {
                        match command {
                            Ok(ProcessorCommand::Pause) => {
                                self.paused = true;
                                tracing::info!("{}: Paused", self.processor_name);
                            },
                            Ok(ProcessorCommand::Shutdown) => {
                                tracing::info!("{}: Shutting down", self.processor_name);
                                break;
                            },
                            Ok(ProcessorCommand::InMemoryMessage(packet)) => {
                                process_packet_func(packet);
                            },
                            _ => {
                                tracing::info!("{}: No command", self.processor_name);
                            }
                        }
                    }
                }
            }
        }
    }
}
