use crate::consensus_engine::block::block::Block;

pub trait Storage {
    fn get_last_block(&self) -> Option<&Block>;
    fn get_block(&self, hash: &Block) -> Option<&Block>;
    fn get_chain(&self, block: &Block) -> Vec<Block>;
    fn save(&self, block: &Block);

    // a partir de cierto bloque queres reemplazar la chain
    fn replace_chain(&self, block: Block, chain: Vec<Block>) -> Vec<Block>;
}