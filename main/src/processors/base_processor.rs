use tokio::sync::mpsc;
use uuid::Uuid;

use std::collections::HashMap;

const MAX_DATA_PER_PACKET_BYTES: usize = 10;

pub trait Processor {
    fn new(name: String) -> Self;
    async fn process(&self, receiver: Option<mpsc::Receiver<Packet>>);
    async fn start(&mut self);
    async fn stop(&mut self);
}

#[derive(Debug)]
pub struct Packet {
    pub data: [u16; MAX_DATA_PER_PACKET_BYTES],
    pub atributes: HashMap<String, String>,
    pub uuid: Uuid,
}

impl Packet {
    pub fn new(data: [u16; MAX_DATA_PER_PACKET_BYTES], atributes: HashMap<String, String>) -> Self {
        Packet {
            data,
            atributes,
            uuid: Uuid::new_v4(),
        }
    }
    pub fn clone(&self) -> Self {
        Packet {
            data: self.data,
            atributes: self.atributes.clone(),
            uuid: self.uuid,
        }
    }
}

#[derive(Debug)]
pub enum ProcessorStatus {
    Running,
    Stopped,
}
