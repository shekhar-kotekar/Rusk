use std::time::Duration;

use processors::{
    in_memory_processor::InMemoryProcessor,
    models::{InMemoryPacket, ProcessorCommand},
};
use tokio::{signal, sync::broadcast, time::sleep};
use uuid::Uuid;

mod processors;

#[tokio::main]
async fn main() {
    commons::enable_tracing();
    let (tx, rx) = broadcast::channel::<ProcessorCommand>(10);
    let mut adder_processor = InMemoryProcessor::new("Adder".to_string(), tx.clone(), rx);
    let mut doubler_processor =
        InMemoryProcessor::new("Doubler".to_string(), tx.clone(), tx.subscribe());

    let adder_handle = tokio::spawn(async move {
        adder_processor.run(adder_func).await;
    });
    let doubler_handle = tokio::spawn(async move {
        doubler_processor.run(doubler_func).await;
    });

    tx.send(ProcessorCommand::InMemoryMessage(InMemoryPacket {
        id: Uuid::new_v4(),
        data: vec![1, 2, 3],
    }));

    tx.send(ProcessorCommand::InMemoryMessage(InMemoryPacket {
        id: Uuid::new_v4(),
        data: vec![4, 5, 6],
    }));

    tx.send(ProcessorCommand::Pause);
    sleep(Duration::from_secs(2)).await;
    tx.send(ProcessorCommand::Resume);

    tx.send(ProcessorCommand::InMemoryMessage({
        InMemoryPacket {
            id: Uuid::new_v4(),
            data: vec![7, 8, 9],
        }
    }));

    match signal::ctrl_c().await {
        Ok(_) => {
            tracing::info!("Ctrl-C received, shutting down");
            tx.send(ProcessorCommand::Shutdown);
            tokio::join!(adder_handle, doubler_handle);
        }
        Err(e) => {
            tracing::error!("Error: {:?}", e);
        }
    }
}

fn adder_func(packet: InMemoryPacket) -> InMemoryPacket {
    let new_data = packet.data.iter().map(|x| x + 1).collect();
    let new_packet = InMemoryPacket {
        id: packet.id,
        data: new_data,
    };
    tracing::info!(
        "old data: {:?}, new data: {:?}",
        packet.data,
        new_packet.data
    );
    new_packet
}

fn doubler_func(packet: InMemoryPacket) -> InMemoryPacket {
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
    new_packet
}
