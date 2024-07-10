use super::models::ProcessorCommand;
use tokio::sync::mpsc;

pub trait Processor {
    fn new(processor_name: String, rx: mpsc::Receiver<ProcessorCommand>) -> Self;

    fn add_tx(&mut self, tx: mpsc::Sender<ProcessorCommand>);
}
