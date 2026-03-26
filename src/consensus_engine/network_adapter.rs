use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::block::Block;
use crate::transaction::Transaction;
use crate::consensus_engine::traits::network::Network as ConsensusNetwork;
use crate::network_layer::types::network::Network;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Message {
    Block(Block),
    Chain(Vec<Block>),
    Transaction(Transaction),
}

pub struct NetworkAdapter {
    network: Arc<Mutex<Network>>,
}

impl NetworkAdapter {
    pub fn new(network: Arc<Mutex<Network>>) -> Self {
        Self { network }
    }
}

impl ConsensusNetwork for NetworkAdapter {
    fn broadcast_block(&self, block: &Block) {
        let msg = serde_json::to_string(&Message::Block(block.clone()))
            .unwrap_or_default();
        self.network.lock().unwrap().broadcast(msg + "\n");
    }

    fn broadcast_chain(&self, chain: Vec<Block>) {
        let msg = serde_json::to_string(&Message::Chain(chain))
            .unwrap_or_default();
        self.network.lock().unwrap().broadcast(msg + "\n");
    }
}
