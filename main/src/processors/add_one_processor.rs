use std::collections::HashMap;

use tokio::sync::mpsc;
use uuid::Uuid;

use super::base_processor::{Packet, Processor, ProcessorStatus};

pub struct AddOneProcessor {
    pub name: String,
    pub uuid: Uuid,
    pub tx: Vec<mpsc::Sender<Packet>>,
    pub status: ProcessorStatus,
}

impl Processor for AddOneProcessor {
    fn new(name: String) -> Self {
        AddOneProcessor {
            name,
            uuid: Uuid::new_v4(),
            tx: Vec::new(),
            status: ProcessorStatus::Running,
        }
    }

    async fn process(&self, receiver: Option<mpsc::Receiver<Packet>>) {
        let mut rx = receiver.unwrap();
        while let Some(packet) = rx.recv().await {
            tracing::info!("Received: {:?}", packet);

            let new_data = packet.data.map(|x| x + 1);
            let packet = Packet::new(new_data, HashMap::new());

            tracing::info!("Processed : {:?}", packet);

            for tx in &self.tx {
                tx.send(packet.clone()).await.unwrap();
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
