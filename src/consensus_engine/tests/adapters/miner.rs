use std::ops::Add;

use crate::{
    common::{
        ports::hasher::Hasher,
        types::{block::Block, tx::Hash},
    },
    consensus_engine::ports::miner::Miner,
};

pub struct CorretlyMinedBlockMiner();

impl Miner for CorretlyMinedBlockMiner {
    fn mine(&self, block: Block, difficulty: usize) -> Block {
        let new_nonce = "0".repeat(difficulty).add(&block.header.nonce.to_string());
        Block::new(
            block.header.version,
            block.header.prev_hash,
            new_nonce.parse().unwrap(),
            block.txs,
            &ZeroHasher(),
        )
    }
}

pub struct ZeroHasher();

impl Hasher for ZeroHasher {
    fn hash(&self, _data: &[u8]) -> Hash {
        Hash([0; 32])
    }
}
