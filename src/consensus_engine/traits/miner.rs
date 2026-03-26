use crate::consensus_engine::block::block::Block;

pub trait Miner {
    fn mine(&self, block: Block, difficulty: usize) -> Block;
}