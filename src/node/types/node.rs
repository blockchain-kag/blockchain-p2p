use crate::{
    consensus_engine::types::consensus_engine::ConsensusEngine, mempool::types::mempool::Mempool,
    network_layer::Network, storage::ports::storage::Storage,
};

pub struct Node {
    network: Network,
    mempool: Mempool,
    consensus_engine: ConsensusEngine,
    storage: Box<dyn Storage>,
}

impl Node {
    pub fn new(
        network: Network,
        mempool: Mempool,
        consensus_engine: ConsensusEngine,
        storage: Box<dyn Storage>,
    ) -> Self {
        Self {
            network,
            mempool,
            consensus_engine,
            storage,
        }
    }

    pub fn run(&self) {
        println!("Starting node...");
        loop {
            println!("Running node...");
        }
    }
}
