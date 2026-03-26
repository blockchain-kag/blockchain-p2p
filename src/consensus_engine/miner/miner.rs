use crate::consensus_engine::block::block::Block;
use crate::consensus_engine::traits::miner::Miner;
use crate::consensus_engine::traits::hasher::{
    Hasher,
    Sha256Hasher
};

pub struct BlockMiner;

impl Miner for BlockMiner {
    fn mine(&self, mut block: Block, difficulty: usize) -> Block {
        let target = "0".repeat(difficulty);

        loop {
            let hash = Sha256Hasher::hash_block(&block);
            if &hash[..difficulty] == target {
                block.hash = hash;
                break;
            }
            block.nonce += 1;
        }

        println!("Block mined: {}", block.hash);
        block
    }
}