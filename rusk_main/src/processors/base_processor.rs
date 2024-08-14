use super::models::ProcessorCommand;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub trait Processor {
    fn new(
        processor_name: String,
        main_comms_sender: mpsc::Sender<ProcessorCommand>,
        rx: mpsc::Receiver<ProcessorCommand>,
    ) -> Self;

    fn add_tx(&mut self, tx: mpsc::Sender<ProcessorCommand>);
}

pub trait ProcessorV2 {
    fn new(
        processor_name: String,
        parent_tx: mpsc::Sender<ProcessorCommand>,
        parent_rx: mpsc::Receiver<ProcessorCommand>,
        cancellation_token: CancellationToken,
        peers_rx: mpsc::Receiver<ProcessorCommand>,
    ) -> Self;

    fn connect_processor(
        &mut self,
        receiver_processor_id: Uuid,
        tx: mpsc::Sender<ProcessorCommand>,
    );
    fn disconnect_processor(&mut self, receiver_processor_id: Uuid);
}
