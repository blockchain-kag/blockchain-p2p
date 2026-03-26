use crate::block::Block;
use crate::consensus_engine::traits::miner::Miner;

pub struct BlockMiner;

impl Miner for BlockMiner {
    fn mine(&self, mut block: Block, difficulty: usize) -> Block {
        let target = "0".repeat(difficulty);
        loop {
            let hash = block.calculate_hash();
            if hash.starts_with(&target) {
                block.hash = hash;
                break;
            }
            block.nonce += 1;
        }
        println!("Block mined: {}", block.hash);
        block
    }
}
