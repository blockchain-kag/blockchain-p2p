use std::default;

use crate::network_layer::traits::{NetworkReceiver, NetworkSender};

pub struct Network {
    peers: Vec<String>,
    sender: Box<dyn NetworkSender>,
    receiver: Box<dyn NetworkReceiver>,
}

impl Network {
    pub fn new(sender: Box<dyn NetworkSender>, receiver: Box<dyn NetworkReceiver>) -> Self {
        Self {
            peers: vec![],
            sender,
            receiver,
        }
    }
    pub fn peers_amount(&self) -> usize {
        self.peers.len()
    }
    pub fn has_peers(&self) -> bool {
        !self.peers.is_empty()
    }
    pub fn has_peer(&self, identifier: String) -> bool {
        self.peers.contains(&identifier)
    }
    pub fn add_peer(&mut self, identifier: String) {
        if !self.peers.contains(&identifier) {
            self.peers.push(identifier);
        }
    }
    pub fn remove_peer(&mut self, identifier: String) {
        self.peers.retain(|peer| *peer != identifier);
    }
    pub fn send_msg(&self, identifier: String, msg: String) -> Result<(), String> {
        self.sender.send(identifier, msg)
    }
    pub fn receive_msg(&self) -> Option<(String, String)> {
        self.receiver.receive()
    }
    pub fn broadcast(&self, msg: String) -> usize {
        let mut amount_sent = 0;
        for peer_id in &self.peers {
            if self.send_msg(peer_id.clone(), msg.clone()).is_ok() {
                amount_sent += 1;
            }
        }
        amount_sent
    }
}

pub struct MockNetworkSender {
    is_result_correct: bool,
}

impl MockNetworkSender {
    pub fn new(is_result_correct: bool) -> MockNetworkSender {
        MockNetworkSender { is_result_correct }
    }
}

impl NetworkSender for MockNetworkSender {
    fn send(&self, _: String, _: String) -> Result<(), String> {
        if self.is_result_correct {
            Ok(())
        } else {
            Err(String::from("Error sending msg"))
        }
    }
}

pub struct MockNetworkReceiver {
    has_received: bool,
}

impl MockNetworkReceiver {
    pub fn new(has_received: bool) -> MockNetworkReceiver {
        Self { has_received }
    }
}

impl NetworkReceiver for MockNetworkReceiver {
    fn receive(&self) -> Option<(String, String)> {
        if self.has_received {
            Some((String::from("sender"), String::from("msg")))
        } else {
            None
        }
    }
}
