use std::collections::HashMap;

use tokio::sync::mpsc;
use uuid::Uuid;

use super::base_processor::{Packet, Processor, ProcessorStatus};

#[derive(Debug)]
pub struct RandomNumberGeneratorProcessor {
    pub name: String,
    pub uuid: Uuid,
    pub tx: HashMap<String, mpsc::Sender<Packet<u16>>>,
    pub status: ProcessorStatus,
    pub sleep_time_milliseconds: u16,
}

impl Processor<u16> for RandomNumberGeneratorProcessor {
    fn new(name: String) -> Self {
        RandomNumberGeneratorProcessor {
            name,
            uuid: Uuid::new_v4(),
            tx: HashMap::new(),
            status: ProcessorStatus::Running,
            sleep_time_milliseconds: 1000,
        }
    }

    async fn process(&mut self, _receiver: Option<mpsc::Receiver<Packet<u16>>>) {
        loop {
            let random_number = rand::random::<u16>();
            let packet = Packet::new(random_number, HashMap::new());
            tracing::info!("{} generated: {:?}", self.name, packet);

            for (key, sender) in self.tx.iter() {
                sender.send(packet.clone()).await.unwrap();
                tracing::info!("Sent packet to {}", key);
            }
            tokio::task::yield_now().await;
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
