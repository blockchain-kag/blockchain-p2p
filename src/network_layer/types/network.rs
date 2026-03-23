use super::super::traits::{NetworkReceiver, NetworkSender};

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
