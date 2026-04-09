use std::sync::mpsc::SendError;

#[derive(Debug)]
pub enum MiningPoolError {
    MinerError(String),
}

impl From<String> for MiningPoolError {
    fn from(value: String) -> Self {
        MiningPoolError::MinerError(value)
    }
}

impl<T> From<SendError<T>> for MiningPoolError {
    fn from(value: SendError<T>) -> Self {
        MiningPoolError::MinerError(format!("Failed to send miner command: {value:?}"))
    }
}
