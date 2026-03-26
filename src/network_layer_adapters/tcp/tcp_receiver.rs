use std::{
    collections::{HashMap, VecDeque},
    io::{ErrorKind, Read},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, channel},
    },
    thread::{self, sleep},
    time::Duration,
};

use crate::network_layer::ports::network_receiver::NetworkReceiver;

struct TcpReceiver {
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

                            let tx_clone = tx.clone();
                            thread::spawn(move || {
                                let mut buffer = [0u8; 1024];
                                let mut stream = stream;

                                loop {
                                    match stream.read(&mut buffer) {
                                        Ok(0) => break,
                                        Ok(n) => {
                                            let msg =
                                                String::from_utf8_lossy(&buffer[..n]).to_string();
                                            tx_clone.send((addr, msg)).ok();
                                        }
                                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                                            sleep(Duration::from_millis(10));
                                        }
                                        Err(_) => break,
                                    }
                                }
                            });
                        } else {
                            break;
                        }
                    }
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(50));
                    }
                    Err(e) => panic!("{}", e),
                }
            }
        });

        Self {
            listener,
            peers,
            rx,
        }
    }
}

impl NetworkReceiver for TcpReceiver {
    fn receive(&mut self) -> Option<(String, String)> {
        None
    }
}
