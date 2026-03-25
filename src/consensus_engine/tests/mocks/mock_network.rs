use crate::consensus_engine::block::block::Block;
use crate::consensus_engine::traits::network::Network;

pub struct MockNetwork {
    pub broadcasted: Vec<Block>,
}

impl MockNetwork {
    pub fn new() -> Self {
        Self { broadcasted: vec![] }
    }
}

impl Network for MockNetwork {
    fn broadcast_block(&self, block: &Block) {
        todo!()
    }

    fn broadcast_chain(&self, blockchain: &[Block]) {
        todo!()
    }
}