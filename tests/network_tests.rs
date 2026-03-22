use blockchain_p2p::network_layer::Network;

#[test]
fn network_test() {
    let mut network = Network::new();
    assert!(!network.has_peers());
    network.add_peer(String::from("a"));
    assert!(network.has_peers())
}
