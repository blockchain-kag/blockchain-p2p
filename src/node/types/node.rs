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
