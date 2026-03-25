use crate::consensus_engine::block::block::Block;
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
    fn get_last_block(&self) -> Option<&Block> {
        todo!()
    }

    fn get_block(&self, hash: &Block) -> Option<&Block> {
        todo!()
    }

    fn get_chain(&self) -> Vec<Block> {
        todo!()
    }

    fn save(&self, block: &Block) {
        todo!()
    }

    fn replace_chain(&self, chain: &Vec<Block>) {
        todo!()
    }
}

