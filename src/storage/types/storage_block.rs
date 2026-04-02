use crate::{common::types::tx::Hash, storage::types::storage_tx::StorageTx};

struct StorageBlockHeader {
    pub version: u32,
    pub prev_hash: Hash,
    pub nonce: u64,
    pub timestamp: u64,
    pub merkle_root: Hash,
}

pub struct StorageBlock {
    header: StorageBlockHeader,
    txs: Vec<StorageTx>,
}

impl StorageBlock {
    pub fn new(
        version: u32,
        prev_hash: Hash,
        nonce: u64,
        timestamp: u64,
        merkle_root: Hash,
        txs: Vec<StorageTx>,
    ) -> Self {
        Self {
            header: StorageBlockHeader {
                version,
                prev_hash,
                nonce,
                timestamp,
                merkle_root,
            },
            txs,
        }
    }
}
