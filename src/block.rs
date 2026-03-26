use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::transaction::Transaction;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub index: u64,
    pub timestamp: u128,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub nonce: u64,
    pub hash: String,
}

impl Block {
    pub fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String) -> Self {
        let timestamp = Self::current_timestamp();
        let mut block = Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            nonce: 0,
            hash: String::new(),
        };
        block.hash = block.calculate_hash();
        block
    }

    fn current_timestamp() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }

    pub fn calculate_hash(&self) -> String {
        let transactions_json = serde_json::to_string(&self.transactions).unwrap_or_default();
        let data = format!(
            "{}{}{}{}{}",
            self.index, self.timestamp, transactions_json, self.previous_hash, self.nonce
        );
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn update_hash(&mut self) {
        self.hash = self.calculate_hash();
    }
}
