use crate::{consensus_engine::types::engine::Engine, mempool::Mempool, network_layer::Network};

pub struct Node {
    network_layer: Network,
    mempool: Mempool,
}

impl Node {
    pub fn new(mut network_layer: Network, mempool: Mempool) -> Self {
        network_layer.add_peer(String::from("172.22.48.23:8000"));
        network_layer.add_peer(String::from("172.22.32.200:8000"));
        Self {
            network_layer,
            mempool,
        }
    }

    pub fn broadcast(&self, msg: String) -> usize {
        self.network_layer.broadcast(msg)
    }
}
