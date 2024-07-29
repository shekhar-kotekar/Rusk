use std::collections::HashMap;

use processors::{
    base_processor::Processor,
    in_memory_processor::InMemoryProcessor,
    in_memory_source_processor::InMemorySourceProcessor,
    models::{InMemoryPacket, ProcessorCommand},
};
use rand::Rng;
use tokio::{signal, sync::mpsc};
use uuid::Uuid;

mod processors;

#[tokio::main]
async fn main() {
    commons::enable_tracing();
    let config = commons::get_config();
    let main_config = config.rusk_main;

    let mut processor_tx: HashMap<Uuid, mpsc::Sender<ProcessorCommand>> = HashMap::new();

    let (tx_for_adder, rx_for_adder) =
        mpsc::channel::<ProcessorCommand>(main_config.processor_queue_length);

    let mut adder_processor =
        InMemorySourceProcessor::new("Adder processor".to_string(), rx_for_adder);

    processor_tx.insert(adder_processor.processor_id, tx_for_adder);

    let (tx_for_doubler, rx_for_doubler) =
        mpsc::channel::<ProcessorCommand>(main_config.processor_queue_length);

    let mut doubler_processor = InMemoryProcessor::new("Doubler".to_string(), rx_for_doubler);
    adder_processor.add_tx(tx_for_doubler.clone());
    processor_tx.insert(doubler_processor.processor_id, tx_for_doubler);

    let adder_handle = tokio::spawn(async move {
        adder_processor.run(adder_func).await;
    });
    let doubler_handle = tokio::spawn(async move {
        doubler_processor.run(doubler_func).await;
    });

    for (_, tx) in processor_tx.iter() {
        tx.send(ProcessorCommand::Start).await.unwrap();
    }

    match signal::ctrl_c().await {
        Ok(_) => {
            tracing::info!("Ctrl-C received, shutting down");
            for (_, tx) in processor_tx {
                tx.send(ProcessorCommand::Shutdown).await.unwrap();
            }
            let result = tokio::join!(adder_handle, doubler_handle);
            match result {
                (Ok(_), Ok(_)) => {
                    tracing::info!("All processors have shut down");
                }
                _ => {
                    tracing::error!("Error shutting down processors");
                }
            }
        }
        Err(e) => {
            tracing::error!("Error: {:?}", e);
        }
    }
}

fn adder_func() -> InMemoryPacket {
    let mut rng = rand::thread_rng();
    let data: Vec<u8> = (0..3).map(|_| rng.gen_range(1..100)).collect();
    InMemoryPacket {
        id: Uuid::new_v4(),
        data,
    }
}

fn doubler_func(packet: InMemoryPacket) -> Option<InMemoryPacket> {
    let new_data = packet.data.iter().map(|x| x * 2).collect();
    let new_packet = InMemoryPacket {
        id: packet.id,
        data: new_data,
    };
    tracing::info!(
        "old data: {:?}, new data: {:?}",
        packet.data,
        new_packet.data
    );
    Some(new_packet)
}
