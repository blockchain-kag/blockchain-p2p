use crate::block::Block;

pub trait Miner: Send {
    fn mine(&self, block: Block, difficulty: usize) -> Block;
}
