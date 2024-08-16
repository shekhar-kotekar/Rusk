use crate::processors::models::ProcessorStatus;

#[derive(PartialEq, Debug, serde::Deserialize, serde::Serialize)]
pub struct ClusterInfo {
    pub cluster_name: String,
    pub processors: Vec<ProcessorInfo>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RequestDetails {
    pub processor_name: String,
    pub processor_id: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ProcessorConnectionRequest {
    pub source_processor_id: String,
    pub destination_processor_id: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ResponseDetails {
    pub processor_id: String,
    pub status: ProcessorStatus,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct ProcessorInfo {
    pub processor_id: String,
    pub status: ProcessorStatus,
    pub packets_processed_count: u64,
}
