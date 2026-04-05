use crate::{
    common::ports::hasher::Hasher,
    common::types::tx::{Hash, Tx},
};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Clone)]
pub struct BlockHeader {
    pub version: u32,
    pub prev_hash: Hash,
    pub nonce: u64,
    pub merkle_root: Hash,
    pub timestamp: u64,
}

impl BlockHeader {
    pub fn hash(&self, hasher: &dyn Hasher) -> Hash {
        let mut data = Vec::new();

        data.extend_from_slice(&self.version.to_be_bytes());
        data.extend_from_slice(&self.prev_hash.0);
        data.extend_from_slice(&self.nonce.to_be_bytes());
        data.extend_from_slice(&self.merkle_root.0);
        data.extend_from_slice(&self.timestamp.to_be_bytes());

        hasher.hash(&data)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub txs: Vec<Tx>,
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

impl Block {
    pub fn new(
        version: u32,
        prev_hash: Hash,
        nonce: u64,
        txs: Vec<Tx>,
        hasher: &dyn Hasher,
    ) -> Self {
        let merkle_root = Self::generate_merkle_root(hasher, &txs);
        Block {
            header: BlockHeader {
                version,
                prev_hash,
                nonce,
                merkle_root,
                timestamp: current_timestamp(),
            },
            txs,
        }
    }

    pub fn new_with_merkle_root(
        version: u32,
        prev_hash: Hash,
        nonce: u64,
        txs: Vec<Tx>,
        merkle_root: Hash,
    ) -> Self {
        Block {
            header: BlockHeader {
                version,
                prev_hash,
                nonce,
                merkle_root,
                timestamp: current_timestamp(),
            },
            txs,
        }
    }

    fn generate_merkle_root(hasher: &dyn Hasher, txs: &[Tx]) -> Hash {
        let mut hashes: Vec<Hash> = txs.iter().map(|tx| hasher.hash(&tx.to_bytes())).collect();

        if hashes.is_empty() {
            return hasher.hash(&[]);
        }

        while hashes.len() > 1 {
            if hashes.len() % 2 == 1 {
                let last = hashes.last().unwrap().clone();
                hashes.push(last);
            }

            let mut next = Vec::new();

            for pair in hashes.chunks(2) {
                let mut data = Vec::new();
                data.extend_from_slice(&pair[0].0);
                data.extend_from_slice(&pair[1].0);

                next.push(hasher.hash(&data));
            }

            hashes = next;
        }

        hashes[0].clone()
    }

    pub fn hash(&self, hasher: &dyn Hasher) -> Hash {
        self.header.hash(hasher)
    }
}
