#[derive(Default)]
pub struct Network {
    peers: Vec<String>,
}

impl Network {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn has_peers(&self) -> bool {
        !self.peers.is_empty()
    }
    pub fn add_peer(&mut self, identifier: String) {
        self.peers.push(identifier);
    }
}
