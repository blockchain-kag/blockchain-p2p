use std::time::{Duration, Instant};

use crate::common::types::tx::Hash;

#[derive(Debug, Clone, Default)]
pub enum MinerState {
    Mining,
    Paused,
    #[default]
    Stopped,
}

impl From<MinerState> for String {
    fn from(value: MinerState) -> Self {
        match value {
            MinerState::Mining => "Mining".to_string(),
            MinerState::Paused => "Paused".to_string(),
            MinerState::Stopped => "Stopped".to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MinerData {
    pub state: MinerState,
    pub hash_rate: Option<f64>,
    pub difficulty: usize,
    pub elapsed: Duration,

    pub current_block_hash: Option<Hash>,
    pub current_nonce: Option<u64>,
    pub attempts: Option<u64>,

    pub last_block_hash: Option<Hash>,
}

impl MinerData {
    pub fn new(difficulty: usize) -> Self {
        MinerData {
            difficulty,
            ..Default::default()
        }
    }
}
