use uuid::Uuid;

#[tokio::main]
async fn main() {
    commons::enable_tracing();
    let first_processor = Processor::new("First Processor".to_string());
    tracing::info!("{:?}", first_processor);
    first_processor.process().await;
}

#[derive(Debug)]
struct Processor {
    name: String,
    uuid: Uuid,
}

impl Processor {
    fn new(name: String) -> Self {
        Processor {
            name,
            uuid: Uuid::new_v4(),
        }
    }

    async fn process(&self) {
        tracing::info!(
            "Processing with name: {} and uuid: {}",
            self.name,
            self.uuid
        );
    }
}
