use serde::{Deserialize, Serialize};

use crate::{
    consensus_engine::{ports::hasher::Hasher, types::tx::Tx},
    mempool::types::tx::Hash,
};

#[derive(Serialize, Deserialize)]
pub struct BlockHeader {
    pub prev_hash: Hash,
    pub nonce: u64,
    pub merkle_root: Hash,
}

#[derive(Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub txs: Vec<Tx>,
}

impl Block {
    pub fn new_generating_merkle_root(
        nonce: u64,
        txs: Vec<Tx>,
        prev_hash: Hash,
        hasher: Box<dyn Hasher>,
    ) -> Self {
        let merkle_root = Self::generate_merkle_root(hasher, txs.clone());
        Block {
            header: BlockHeader {
                prev_hash,
                nonce,
                merkle_root,
            },
            txs,
        }
    }

    pub fn new(nonce: u64, txs: Vec<Tx>, prev_hash: Hash, merkle_root: Hash) -> Self {
        Block {
            header: BlockHeader {
                prev_hash,
                nonce,
                merkle_root,
            },
            txs,
        }
    }

    fn generate_merkle_root(hasher: Box<dyn Hasher>, txs: Vec<Tx>) -> Hash {
        let mut hashes: Vec<Hash> = txs
            .into_iter()
            .map(|tx| hasher.hash(&tx.to_bytes()))
            .collect();

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
}
