use crate::block::Block;

pub trait Network: Send {
    fn broadcast_block(&self, block: &Block);
    fn broadcast_chain(&self, chain: Vec<Block>);
}
