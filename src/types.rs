use libsecp256k1::SecretKey;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

pub const DIFFICULTY: usize = 4;
pub const BLOCK_REWARD: u64 = 10;
pub const AUTO_MINE_THRESHOLD: usize = 3;
pub const GENESIS_TIMESTAMP: u64 = 1743086300000;
pub const ZEROS: &str = "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    #[serde(rename = "type")]
    pub tx_type: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub timestamp: u64,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    #[serde(rename = "previousHash")]
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
}

pub struct Wallet {
    pub secret_key: SecretKey,
    pub public_key_hex: String,
    pub address: String,
}

pub struct AppState {
    pub chain: Vec<Block>,
    pub mempool: Vec<Transaction>,
    pub peers: HashSet<String>,
    pub wallet: Wallet,
    pub node_url: String,
}

pub type Shared = Arc<Mutex<AppState>>;
