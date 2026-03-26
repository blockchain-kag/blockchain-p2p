use crate::block::Block;
use crate::consensus_engine::traits::storage::Storage;

pub struct MockStorage {
    pub saved: Vec<Block>,
}

impl MockStorage {
    pub fn new() -> Self {
        MockStorage { saved: vec![] }
    }
}

impl Storage for MockStorage {
    fn get_last_block(&self) -> Option<Block> {
        self.saved.last().cloned()
    }

    fn get_block(&self, block: &Block) -> Option<Block> {
        self.saved.iter().find(|b| b.hash == block.hash).cloned()
    }

    fn get_chain(&self, _block: &Block) -> Vec<Block> {
        self.saved.clone()
    }

    fn save(&self, _block: &Block) {
        // mock — no-op
    }

    fn replace_chain(&self, _block: Block, chain: Vec<Block>) -> Vec<Block> {
        chain
    }
}
