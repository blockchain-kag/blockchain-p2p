use crate::{
    consensus_engine::types::consensus_engine::ConsensusEngine,
    events::ports::event_stream::EventStream, mempool::types::mempool::Mempool,
    storage::ports::storage::Storage,
};

pub struct Node {
    event_streams: Vec<Box<dyn EventStream>>,
    mempool: Mempool,
    consensus_engine: ConsensusEngine,
    storage: Box<dyn Storage>,
}

impl Node {
    pub fn new(
        event_streams: Vec<Box<dyn EventStream>>,
        mempool: Mempool,
        consensus_engine: ConsensusEngine,
        storage: Box<dyn Storage>,
    ) -> Self {
        Self {
            event_streams,
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
