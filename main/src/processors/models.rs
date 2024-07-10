use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum ProcessorStatus {
    Paused,
    Running,
    Stopped,
}

#[derive(Clone, Debug)]
pub enum ProcessorCommand {
    Start,
    Pause,
    Resume,
    Shutdown,
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
