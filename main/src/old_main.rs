use processors::{
    add_one_processor::AddOneProcessor,
    base_processor::{Packet, Processor},
    random_number_generator::RandomNumberGeneratorProcessor,
};
use tokio::sync::mpsc;

mod models;
mod processors;

const PROCESSOR_DEFAULT_QUEUE_LENGTH: usize = 100;

#[tokio::main]
async fn main() {
    commons::enable_tracing();
    let (tx, rx) = mpsc::channel::<Packet<u16>>(PROCESSOR_DEFAULT_QUEUE_LENGTH);
    let mut random_number_generator_processor =
        RandomNumberGeneratorProcessor::new("R1".to_string());

    random_number_generator_processor
        .tx
        .insert("AddOne".to_string(), tx.clone());

    let mut add_one_processor = AddOneProcessor::new("Add One".to_string());

    tokio::spawn(async move {
        random_number_generator_processor.process(None).await;
    });

    let mut r2 = RandomNumberGeneratorProcessor::new("R2".to_string());
    r2.tx.insert("AddOne".to_string(), tx.clone());

    tokio::spawn(async move {
        r2.process(None).await;
    });

    tokio::spawn(async move {
        add_one_processor.process(Some(rx)).await;
    });
    tracing::info!("task spawned for AddOneProcessor");

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
