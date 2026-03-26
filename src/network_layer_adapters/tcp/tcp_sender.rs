use std::{
    collections::HashMap,
    io::Write,
    net::{SocketAddr, TcpStream},
    sync::Mutex,
};

use crate::network_layer::ports::network_sender::NetworkSender;

pub struct TCPSender {
    peers: Mutex<HashMap<SocketAddr, TcpStream>>,
}

impl TCPSender {
    pub fn new() -> Self {
        Self {
            peers: Mutex::new(HashMap::new()),
        }
    }
}

impl NetworkSender for TCPSender {
    fn send(&self, identifier: String, msg: String) -> Result<String, String> {
        let mut peers = self.peers.lock().unwrap();

        let socket_addr = identifier
            .parse::<SocketAddr>()
            .map_err(|e| format!("Invalid peer address: {} ({})", identifier, e))?;

        let stream = match peers.entry(socket_addr) {
            std::collections::hash_map::Entry::Occupied(e) => e.into_mut(),
            std::collections::hash_map::Entry::Vacant(e) => {
                let conn = TcpStream::connect(socket_addr)
                    .map_err(|e| format!("Connection failed to {}: {}", socket_addr, e))?;
                e.insert(conn)
            }
        };

        stream
            .write_all(msg.as_bytes())
            .map_err(|e| format!("Send error: {}", e))?;

        Ok(String::from("Sent"))
    }
}
