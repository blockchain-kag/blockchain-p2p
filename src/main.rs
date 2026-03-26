use std::io::{self, BufRead};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use blockchain_p2p::{
    consensus_engine::{
        BlockMiner, Engine, InMemoryStorage, Message, NetworkAdapter,
    },
    mempool::Mempool,
    network_layer::types::network::Network,
    network_layer_adapters::tcp::{tcp_receiver::TcpReceiver, tcp_sender::TCPSender},
    transaction::Transaction,
};

fn get_arg(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .map(|w| w[1].clone())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let port = get_arg(&args, "--port").unwrap_or_else(|| "8001".into());
    let peers_str = get_arg(&args, "--peers").unwrap_or_default();
    let difficulty: usize = get_arg(&args, "--difficulty")
        .unwrap_or_else(|| "3".into())
        .parse()
        .unwrap_or(3);

    let peers: Vec<String> = if peers_str.is_empty() {
        vec![]
    } else {
        peers_str.split(',').map(String::from).collect()
    };

    let addr = format!("0.0.0.0:{}", port);
    let network = Arc::new(Mutex::new(Network::new(
        Box::new(TCPSender::new()),
        Box::new(TcpReceiver::new(&addr)),
    )));

    for peer in &peers {
        network.lock().unwrap().add_peer(peer.clone());
        println!("Peer agregado: {}", peer);
    }

    let storage = Box::new(InMemoryStorage::new());
    let network_adapter = Box::new(NetworkAdapter::new(Arc::clone(&network)));
    let mempool = Box::new(Mempool::new());
    let miner = Box::new(BlockMiner);
    let engine = Arc::new(Mutex::new(Engine::new(
        storage,
        network_adapter,
        mempool,
        miner,
        difficulty,
    )));

    println!("=== Nodo iniciado en puerto {} (dificultad: {}) ===", port, difficulty);
    println!("Comandos: mine | tx <from> <to> <amount> | peers <ip:port> | quit");

    // Hilo receptor de mensajes
    let engine_rx = Arc::clone(&engine);
    let network_rx = Arc::clone(&network);
    thread::spawn(move || loop {
        let msg_opt = network_rx.lock().unwrap().receive_msg();
        if let Some((from, raw)) = msg_opt {
            let raw = raw.trim().to_string();
            match serde_json::from_str::<Message>(&raw) {
                Ok(Message::Block(block)) => {
                    println!("[red] Bloque #{} recibido de {}", block.index, from);
                    engine_rx.lock().unwrap().receive_block(block);
                }
                Ok(Message::Chain(chain)) => {
                    println!("[red] Chain ({} bloques) recibida de {}", chain.len(), from);
                    if let Some(last) = chain.last().cloned() {
                        engine_rx.lock().unwrap().receive_chain(last, chain);
                    }
                }
                Ok(Message::Transaction(tx)) => {
                    println!("[red] Tx recibida de {}: {} -> {} ({})", from, tx.from, tx.to, tx.amount);
                    engine_rx.lock().unwrap().add_transaction(tx);
                }
                Err(_) => {}
            }
        }
        thread::sleep(Duration::from_millis(10));
    });

    // Loop principal — CLI
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        match parts.as_slice() {
            ["mine"] => {
                println!("Minando...");
                engine.lock().unwrap().mine_new_block();
            }
            ["tx", from, to, amount] => {
                let amount: u64 = match amount.parse() {
                    Ok(a) => a,
                    Err(_) => { println!("Monto inválido"); continue; }
                };
                let tx = Transaction::new(
                    from.to_string(),
                    to.to_string(),
                    amount,
                    "sig".to_string(),
                );
                engine.lock().unwrap().add_transaction(tx);
                println!("Tx agregada al mempool");
            }
            ["peers", addr] => {
                network.lock().unwrap().add_peer(addr.to_string());
                println!("Peer agregado: {}", addr);
            }
            ["quit"] => {
                println!("Saliendo...");
                break;
            }
            _ => println!("Comando desconocido. Usá: mine | tx <from> <to> <amount> | peers <ip:port> | quit"),
        }
    }
}
