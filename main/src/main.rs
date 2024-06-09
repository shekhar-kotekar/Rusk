use std::collections::HashMap;

use uuid::Uuid;

const PROCESSOR_DEFAULT_QUEUE_LENGTH: u16 = 50;

#[tokio::main]
async fn main() {
    commons::enable_tracing();

    let mut flow_graph = FlowGraph::new();

    let first_processor = flow_graph.create_processor("Processor A".to_string(), Some(100));
    tracing::info!("{:?}", first_processor);
    first_processor.process().await;

    let processor_b = flow_graph.create_processor("Processor B".to_string(), None);
    tracing::info!("{:?}", processor_b);

    flow_graph.add_edge(&first_processor, &processor_b);
    flow_graph.print_edges();

    first_processor.stop().await;
    processor_b.stop().await;
}

#[derive(Debug)]
struct Processor {
    name: String,
    uuid: Uuid,
    queue_length: u16,
}

impl Processor {
    fn new(name: String, queue_length: u16) -> Self {
        Processor {
            name,
            uuid: Uuid::new_v4(),
            queue_length: queue_length,
        }
    }

    async fn process(&self) {
        tracing::info!(
            "Processing with name: {} and uuid: {}",
            self.name,
            self.uuid
        );
    }

    async fn stop(&self) {
        tracing::info!("Stopping processor with name: {}", self.name);
    }
}

struct FlowGraph {
    edges: HashMap<String, String>,
}

impl FlowGraph {
    fn new() -> Self {
        FlowGraph {
            edges: HashMap::new(),
        }
    }

    fn create_processor(&self, name: String, queue_length: Option<u16>) -> Processor {
        Processor::new(name, queue_length.unwrap_or(PROCESSOR_DEFAULT_QUEUE_LENGTH))
    }

    fn add_edge(&mut self, from: &Processor, to: &Processor) {
        self.edges
            .insert(from.uuid.to_string(), to.uuid.to_string());
    }

    fn print_edges(&self) {
        for (from, to) in self.edges.iter() {
            tracing::info!("Edge from: {} to: {}", from, to);
        }
    }
}
