use tokio::sync::mpsc;
use uuid::Uuid;

use super::base_processor::{Packet, Processor, ProcessorStatus};

pub struct AddOneProcessor {
    pub name: String,
    pub uuid: Uuid,
    pub tx: Vec<mpsc::Sender<Packet<u16>>>,
    pub status: ProcessorStatus,
    pub processed_packet_count: u64,
}

impl Processor<u16> for AddOneProcessor {
    fn new(name: String) -> Self {
        AddOneProcessor {
            name,
            uuid: Uuid::new_v4(),
            tx: Vec::new(),
            status: ProcessorStatus::Running,
            processed_packet_count: 0,
        }
    }

    async fn process(&mut self, receiver: Option<mpsc::Receiver<Packet<u16>>>) {
        let mut rx = receiver.unwrap();
        while let Some(packet) = rx.recv().await {
            tracing::info!("Received: {:?}", packet);
            self.processed_packet_count += 1;
            let processed_packet = Packet::new(packet.data + 1, packet.atributes);

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
