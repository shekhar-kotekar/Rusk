use tokio::sync::mpsc;
use uuid::Uuid;

use std::{collections::HashMap, fmt::Debug};

pub trait Processor<T> {
    fn new(name: String) -> Self;
    async fn process(&mut self, receiver: Option<mpsc::Receiver<Packet<T>>>);
    async fn start(&mut self);
    async fn stop(&mut self);
}

#[derive(Debug, Clone)]
pub struct Packet<T> {
    pub data: T,
    pub atributes: HashMap<String, String>,
    pub uuid: Uuid,
}

impl<T: Debug> Packet<T> {
    pub fn new(data: T, atributes: HashMap<String, String>) -> Self {
        Packet {
            data,
            atributes,
            uuid: Uuid::new_v4(),
        }
    }
}

#[derive(Debug)]
pub enum ProcessorStatus {
    Running,
    Stopped,
}
