use crate::consensus_engine::block::block::Block;
use sha2::{
    Digest,
    Sha256
};


pub trait Hasher {
    fn hash_block(block: &Block) -> String;
}

pub struct Sha256Hasher;

impl Hasher for Sha256Hasher {
    fn hash_block(block: &Block) -> String {
        let input = format!(
            "{}{}{}{}{}",
            block.index,
            block.timestamp,
            serde_json::to_string(&block.transactions).unwrap(),
            block.previous_hash,
            block.nonce
        );
        let mut hasher = Sha256::new();
        hasher.update(input);
        format!("{:x}", hasher.finalize())
    }
}