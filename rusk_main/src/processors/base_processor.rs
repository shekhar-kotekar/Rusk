use super::models::{Message, ProcessorCommand};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub trait ProcessorConnection {
    fn connect_processor(&mut self, receiver_processor_id: Uuid, tx: mpsc::Sender<Message>);
    fn disconnect_processor(&mut self, receiver_processor_id: Uuid);
}

pub trait SourceProcessor {
    fn new(
        processor_name: String,
        parent_rx: mpsc::Receiver<ProcessorCommand>,
        peer_processors_tx: Vec<mpsc::Sender<Message>>,
        cancellation_token: CancellationToken,
    ) -> Self;
}

pub trait SinkProcessor {
    fn new(
        processor_name: String,
        peers_rx: mpsc::Receiver<Message>,
        parent_rx: mpsc::Receiver<ProcessorCommand>,
        cancellation_token: CancellationToken,
    ) -> Self;
}
