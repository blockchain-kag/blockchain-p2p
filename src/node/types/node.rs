use crate::{
    consensus_engine::types::engine::Engine,
    mempool::Mempool,
    network_layer::{self, Network},
};

struct Node {
    network_layer: Network,
    mempool: Mempool,
    consensus_engine: Engine,
}

impl Node {
    fn new(network_layer: Network, mempool: Mempool, engine: Engine) -> Self {
        Self {
            network_layer,
            mempool,
            consensus_engine: engine,
        }
    }
}
