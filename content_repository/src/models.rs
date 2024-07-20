use bytes::Bytes;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum Command {
    Data {
        content: Bytes,
        tx: oneshot::Sender<u64>,
    },
}
