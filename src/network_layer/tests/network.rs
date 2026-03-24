use crate::network_layer::{
    Network,
    mocks::{mock_receiver::MockNetworkReceiver, mock_sender::MockNetworkSender},
};

#[test]
fn create_network_test() {
    let network = Network::new(
        Box::new(MockNetworkSender::new(true)),
        Box::new(MockNetworkReceiver::new(true)),
    );
    assert!(!network.has_peers());
}

#[test]
fn add_peer_to_network_test() {
    let mut network = Network::new(
        Box::new(MockNetworkSender::new(true)),
        Box::new(MockNetworkReceiver::new(true)),
    );
    let identifier = String::from("peer 1");
    network.add_peer(identifier.clone());
    assert!(network.has_peers());
    assert!(network.has_peer(identifier));
    assert!(!network.has_peer(String::from("peer 2")));
}

#[test]
fn does_not_add_duplicate_peers() {
    let mut network = Network::new(
        Box::new(MockNetworkSender::new(true)),
        Box::new(MockNetworkReceiver::new(true)),
    );
    let id = "peer1".to_string();

    network.add_peer(id.clone());
    network.add_peer(id.clone());

    assert_eq!(network.peers_amount(), 1);
}

#[test]
fn remove_peer_from_network_test() {
    let mut network = Network::new(
        Box::new(MockNetworkSender::new(true)),
        Box::new(MockNetworkReceiver::new(true)),
    );
    let identifier_1 = String::from("peer 1");
    let identifier_2 = String::from("peer 2");
    network.add_peer(identifier_1.clone());
    network.add_peer(identifier_2.clone());
    network.remove_peer(identifier_1.clone());
    assert!(!network.has_peer(identifier_1));
    assert!(network.has_peer(identifier_2));
}

#[test]
fn remove_unexistent_peer_from_network_test() {
    let mut network = Network::new(
        Box::new(MockNetworkSender::new(true)),
        Box::new(MockNetworkReceiver::new(true)),
    );
    let identifier = String::from("peer 1");
    network.remove_peer(identifier);
    assert!(!network.has_peers())
}

#[test]
fn send_msg_test() {
    let mut network = Network::new(
        Box::new(MockNetworkSender::new(true)),
        Box::new(MockNetworkReceiver::new(true)),
    );
    let identifier = String::from("peer 1");
    network.add_peer(identifier.clone());
    let msg = String::from("msg");
    assert!(network.send_msg(identifier, msg.clone()).is_ok());
    let network = Network::new(
        Box::new(MockNetworkSender::new(false)),
        Box::new(MockNetworkReceiver::new(true)),
    );
    assert!(
        network
            .send_msg(String::from("peer 1"), msg.clone())
            .is_err()
    );
}
#[test]
fn receive_msg_test() {
    let sender = MockNetworkSender::new(true);
    let mut network = Network::new(Box::new(sender), Box::new(MockNetworkReceiver::new(true)));
    assert!(network.receive_msg().is_some());
    let sender = MockNetworkSender::new(true);
    network = Network::new(Box::new(sender), Box::new(MockNetworkReceiver::new(false)));
    assert!(network.receive_msg().is_none());
}
#[test]
fn broadcast_msg_test() {
    let mut network = Network::new(
        Box::new(MockNetworkSender::new(true)),
        Box::new(MockNetworkReceiver::new(true)),
    );
    assert_eq!(0, network.broadcast(String::from("msg")));
    network.add_peer(String::from("peer 1"));
    assert_eq!(1, network.broadcast(String::from("msg")));
    network.add_peer(String::from("peer 2"));
    assert_eq!(2, network.broadcast(String::from("msg")));
}
#[test]
fn broadcast_counts_only_successful_sends() {
    let mut network = Network::new(
        Box::new(MockNetworkSender::new(false)), // fails
        Box::new(MockNetworkReceiver::new(true)),
    );

    network.add_peer("p1".to_string());
    network.add_peer("p2".to_string());

    assert_eq!(0, network.broadcast("msg".to_string()));
}
