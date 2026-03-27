use std::ptr::null;

use blockchain_p2p::{
    consensus_engine::types::engine::{self, Engine},
    mempool::Mempool,
    network_layer::{self, Network},
    network_layer_adapters::tcp::{tcp_receiver::TcpReceiver, tcp_sender::TCPSender},
    node::types::node::Node,
};

fn main() {
    let receiver = TcpReceiver::new("0.0.0.0:9000");
    let sender = TCPSender::new();
    let network_layer = Network::new(Box::new(sender), Box::new(receiver));
    let mempool = Mempool::new();
    let node = Node::new(network_layer, mempool);
    node.broadcast(String::from("s"));
}
