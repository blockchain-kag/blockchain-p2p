use std::sync::Mutex;

use crate::block::Block;
use crate::consensus_engine::traits::storage::Storage;

pub struct InMemoryStorage {
    chain: Mutex<Vec<Block>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            chain: Mutex::new(Vec::new()),
        }
    }
}

impl Storage for InMemoryStorage {
    fn get_last_block(&self) -> Option<Block> {
        self.chain.lock().unwrap().last().cloned()
    }

    fn get_block(&self, block: &Block) -> Option<Block> {
        self.chain
            .lock()
            .unwrap()
            .iter()
            .find(|b| b.hash == block.hash)
            .cloned()
    }

    fn get_chain(&self, _block: &Block) -> Vec<Block> {
        self.chain.lock().unwrap().clone()
    }

    fn save(&self, block: &Block) {
        self.chain.lock().unwrap().push(block.clone());
    }

    fn replace_chain(&self, _block: Block, chain: Vec<Block>) -> Vec<Block> {
        let mut current = self.chain.lock().unwrap();
        *current = chain.clone();
        chain
    }
}
