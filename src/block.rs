use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Sha256, Digest};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: u128,
    pub transactions: Vec<String>,
    pub previous_hash: String,
    pub nonce: u64,
    pub hash: String,
}

// This code create an initial block
// Then miner.rs change nonce value

impl Block {

    pub fn new(
        index: u64,
        transactions: Vec<String>, // todo: change to Vec<Transaction>
        previous_hash: String,
    ) -> Self {

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

    // This generates a hash SHA256 of block
    pub fn calculate_hash(&self) -> String {
        let data = format!(
            "{}{}{:?}{}{}",
            self.index,
            self.timestamp,
            self.transactions,
            self.previous_hash,
            self.nonce
        );

        let mut hasher = Sha256::new();
        hasher.update(data);

        let result = hasher.finalize();

        format!("{:x}", result)
    }


    // Then mining, nonce chance so we need to recalculate hash value.
    pub fn update_hash(&mut self) {
        self.hash = self.calculate_hash();
    }
}

