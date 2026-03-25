use crate::consensus_engine::transaction::transaction::Transaction;
use serde::{
    Deserialize,
    Serialize
};
use chrono::Utc;
use sha2::{
    Digest,
    Sha256
};
use crate::consensus_engine::traits::hasher::{Hasher, Sha256Hasher};

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub(crate) index: u64,
    pub(crate) timestamp: i64,
    pub(crate) transactions: Vec<Transaction>,
    pub(crate) previous_hash: String,
    pub(crate) hash: String,
    pub(crate) nonce: u64
}

impl Block {
    pub fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String) -> Self {
        let timestamp = Utc::now().timestamp();
        let mut block = Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash: String::new(),
            nonce: 0,
        };
        block.hash = Sha256Hasher::hash_block(&block);
        block
    }
}