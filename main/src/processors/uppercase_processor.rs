use tokio::sync::mpsc;
use uuid::Uuid;

use super::base_processor::{Packet, Processor, ProcessorStatus};

pub struct UppercaseProcessor {
    pub name: String,
    pub uuid: Uuid,
    pub tx: Vec<mpsc::Sender<Packet<String>>>,
    pub status: ProcessorStatus,
    pub processed_packet_count: u64,
}

impl Processor<String> for UppercaseProcessor {
    fn new(name: String) -> Self {
        UppercaseProcessor {
            name,
            uuid: Uuid::new_v4(),
            tx: Vec::new(),
            status: ProcessorStatus::Running,
            processed_packet_count: 0,
        }
    }

    async fn process(&mut self, receiver: Option<mpsc::Receiver<Packet<String>>>) {
        let mut rx = receiver.unwrap();
        while let Some(packet) = rx.recv().await {
            tracing::info!("Received: {:?}", packet);
            self.processed_packet_count += 1;

            let processed_packet = Packet::new(packet.data.to_uppercase(), packet.atributes);

            tracing::info!("Processed : {:?}", processed_packet);

            for tx in &self.tx {
                tx.send(processed_packet.clone()).await.unwrap();
            }
        }
    }

    async fn start(&mut self) {
        tracing::info!("Starting {} processor", self.name);
        self.status = ProcessorStatus::Running;
    }

    async fn stop(&mut self) {
        tracing::info!("Stopping {} processor", self.name);
        self.status = ProcessorStatus::Stopped;
    }
}
