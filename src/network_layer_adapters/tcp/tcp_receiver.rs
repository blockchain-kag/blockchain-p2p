use std::{
    collections::HashMap,
    io::{ErrorKind, Read},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        Arc, Mutex, Weak,
        mpsc::{Receiver, Sender, channel},
    },
    thread::{self, sleep},
    time::Duration,
};

use crate::network_layer::ports::network_receiver::NetworkReceiver;

pub struct TcpReceiver {
    listener: Arc<TcpListener>,
    peers: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
    rx: Receiver<(SocketAddr, String)>,
}

impl TcpReceiver {
    pub fn new(addr: &str) -> Self {
        let listener = Arc::new(TcpListener::bind(addr).unwrap());
        listener.set_nonblocking(true).unwrap();
        let weak_listener = Arc::downgrade(&listener);
        let peers: Arc<Mutex<HashMap<SocketAddr, TcpStream>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let weak_peers = Arc::downgrade(&peers);
        let (tx, rx) = channel();

        spawn_new_peers_listener_thread(weak_listener, weak_peers, tx);

        Self {
            listener,
            peers,
            rx,
        }
    }
}

fn spawn_new_peers_listener_thread(
    weak_listener: Weak<TcpListener>,
    weak_peers: Weak<Mutex<HashMap<SocketAddr, TcpStream>>>,
    tx: Sender<(SocketAddr, String)>,
) {
    thread::spawn(move || {
        loop {
            let listener = match weak_listener.upgrade() {
                Some(l) => l,
                None => break,
            };

            match listener.accept() {
                Ok((stream, addr)) => {
                    println!("New peer: {}", addr);
                    if let Some(peers) = weak_peers.upgrade() {
                        peers
                            .lock()
                            .unwrap()
                            .insert(addr, stream.try_clone().unwrap());

                        spawn_peer_reader_thread(stream, addr, tx.clone());
                    } else {
                        break;
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    sleep(Duration::from_millis(50));
                }
                Err(e) => panic!("{}", e),
            }
        }
    });
}

fn spawn_peer_reader_thread(
    mut stream: TcpStream,
    addr: SocketAddr,
    tx: Sender<(SocketAddr, String)>,
) {
    thread::spawn(move || {
        let mut read_buf = [0u8; 1024];
        let mut buffer: Vec<u8> = Vec::new(); // ← persistent buffer

        loop {
            match stream.read(&mut read_buf) {
                Ok(0) => break,
                Ok(n) => {
                    process_incoming_bytes(addr, &tx, &read_buf, &mut buffer, n);
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    sleep(Duration::from_millis(10));
                }
                Err(_) => break,
            }
        }
    });
}

fn process_incoming_bytes(
    addr: SocketAddr,
    tx: &Sender<(SocketAddr, String)>,
    read_buf: &[u8],
    buffer: &mut Vec<u8>,
    n: usize,
) {
    buffer.extend_from_slice(&read_buf[..n]);

    while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
        let line: Vec<u8> = buffer.drain(..=pos).collect();

        let msg = String::from_utf8_lossy(&line[..line.len() - 1]).to_string();

        tx.send((addr, msg)).ok();
    }
}

impl NetworkReceiver for TcpReceiver {
    fn receive(&mut self) -> Option<(String, String)> {
        match self.rx.try_recv() {
            Ok((addr, msg)) => Some((addr.to_string(), msg)),
            Err(_) => None,
        }
    }
}
