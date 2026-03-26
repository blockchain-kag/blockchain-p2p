use std::{
    collections::HashMap,
    io::Write,
    net::{SocketAddr, TcpStream},
    str::FromStr,
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
    fn send(&self, identifier: String, msg: String) -> Result<(), String> {
        let mut peers = self.peers.lock().unwrap();

        let socket_addr = identifier
            .parse::<SocketAddr>()
            .map_err(|e| format!("Invalid peer address: {} ({})", identifier, e))?;
        let stream = peers
            .entry(socket_addr)
            .or_insert_with(|| TcpStream::connect(socket_addr).expect("connection failed"));

        stream
            .write_all(msg.as_bytes())
            .map_err(|e| format!("Send error: {}", e))?;

        Ok(())
    }
}
