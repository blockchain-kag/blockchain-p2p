use crate::consensus_engine::block::block::Block;

pub trait Storage {
    fn get_last_block(&self) -> Block;
    fn get_block(&self, hash: &Block) -> Option<&Block>;
    fn get_chain(&self) -> Vec<Block>; 
    fn save(&self, block: &Block);
    fn replace_chain(&self, chain: &Vec<Block>);
}