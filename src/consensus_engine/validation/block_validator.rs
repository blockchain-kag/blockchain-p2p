use crate::consensus_engine::block::block::Block;

pub struct BlockValidator;

use crate::consensus_engine::traits::hasher::{
    Hasher,
    Sha256Hasher
};


impl BlockValidator {
    pub fn validate(current: &Block, previous: Option<&Block>) -> bool {
        if previous.is_none() {
            return current.index == 0
                && current.previous_hash.is_empty()
                && current.hash == Sha256Hasher::hash_block(current);
        }

        let previous = previous.unwrap();

        if current.hash != Sha256Hasher::hash_block(current) { return false; }
        if current.previous_hash != previous.hash { return false; }
        if current.index != previous.index + 1 { return false; }
        true
    }
}