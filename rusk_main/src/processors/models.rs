use uuid::Uuid;

#[derive(Copy, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ProcessorStatus {
    Paused,
    Running,
    Stopped,
}

impl PartialEq for ProcessorStatus {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ProcessorStatus::Running, ProcessorStatus::Running) => true,
            (ProcessorStatus::Stopped, ProcessorStatus::Stopped) => true,
            (ProcessorStatus::Paused, ProcessorStatus::Paused) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum ProcessorCommand {
    Stop,
    Pause,
    Resume,
    Start,
    InMemoryMessage(InMemoryPacket),
    ReferenceMessage(ReferencePacket),
}

#[derive(Clone, Debug)]
pub struct InMemoryPacket {
    pub id: Uuid,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct ReferencePacket {
    pub id: Uuid,
    pub file_name: String,
    pub offset: u64,
    pub length: u64,
}
