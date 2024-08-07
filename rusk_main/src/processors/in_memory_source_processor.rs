use std::time::Duration;

use tokio::{sync::mpsc, time::sleep};
use uuid::Uuid;

use super::{
    base_processor::Processor,
    models::{InMemoryPacket, ProcessorCommand, ProcessorStatus},
};

pub struct InMemorySourceProcessor {
    pub processor_name: String,
    pub processor_id: Uuid,
    tx: Vec<mpsc::Sender<ProcessorCommand>>,
    rx: mpsc::Receiver<ProcessorCommand>,
    pub status: ProcessorStatus,
    delay: u64,
}

impl Processor for InMemorySourceProcessor {
    fn new(
        processor_name: String,
        rx: mpsc::Receiver<ProcessorCommand>,
    ) -> InMemorySourceProcessor {
        InMemorySourceProcessor {
            processor_name,
            processor_id: Uuid::new_v4(),
            tx: Vec::new(),
            rx,
            delay: 1000,
            status: ProcessorStatus::Stopped,
        }
    }
    fn add_tx(&mut self, tx: mpsc::Sender<ProcessorCommand>) {
        self.tx.push(tx);
    }
}

impl InMemorySourceProcessor {
    pub async fn run(&mut self, generate_packet_func: fn() -> InMemoryPacket) {
        loop {
            match self.status {
                ProcessorStatus::Running => {
                    if self.tx.is_empty() {
                        tracing::info!(
                            "{}: No processors to send packet to. Pausing.",
                            self.processor_name
                        );
                        self.status = ProcessorStatus::Stopped;
                    } else {
                        let packet = generate_packet_func();
                        for tx in &self.tx {
                            let _ = tx
                                .send(ProcessorCommand::InMemoryMessage(packet.clone()))
                                .await;
                        }
                        let result = self.rx.try_recv();
                        if let Ok(ProcessorCommand::Stop) = result {
                            tracing::info!("{} : processor stopped.", self.processor_name);
                            break;
                        }
                        tracing::info!(
                            "{}: packet sent to {} processors. Sleeping for {} ms.",
                            self.processor_name,
                            self.tx.len(),
                            self.delay
                        );
                        sleep(Duration::from_millis(self.delay)).await;
                    }
                }
                ProcessorStatus::Stopped => {
                    tracing::info!(
                        "{} processor is Stopped. Waiting for command on rx channel.",
                        self.processor_name
                    );
                    let result = self.rx.recv().await;
                    if let Some(ProcessorCommand::Start) = result {
                        self.status = ProcessorStatus::Running;
                        tracing::info!("{}: Started", self.processor_name);
                    }
                }
            }
        }
    }
}
