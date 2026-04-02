use crate::common::types::block::Block;

pub trait Miner {
    fn mine(&self, block: Block, difficulty: usize) -> Block;
}
