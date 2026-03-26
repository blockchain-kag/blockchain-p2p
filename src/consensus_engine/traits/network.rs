use crate::consensus_engine::block::block::Block;

pub trait Network {
    fn broadcast_block(&self, blockchain: &Block);
    fn broadcast_chain(&self, blockchain: Vec<Block>);
}