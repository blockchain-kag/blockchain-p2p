// Blockchain P2P Node
// Usage:
//   cargo run -- <port>                                    Run node
//   cargo run -- wallet                                    Generate wallet
//   cargo run -- send <node_url> <to> <amount> <privkey>   Send transaction
//
// Env:
//   SEED_PEERS=http://ip:port,http://ip:port
//   NODE_URL=http://my_ip:port

mod crypto;
mod mining;
mod network;
mod routes;
mod types;
mod validation;

use axum::routing::{get, post};
use axum::Router;
use libsecp256k1::{PublicKey, SecretKey};
use serde_json::json;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crypto::{now_ms, pubkey_to_address, sign_message};
use mining::create_genesis;
use network::bootstrap;
use types::{AppState, Transaction, Wallet};

fn create_wallet() -> Wallet {
    let mut rng = rand::thread_rng();
    let sk = SecretKey::random(&mut rng);
    let pk = PublicKey::from_secret_key(&sk);
    Wallet {
        secret_key: sk,
        public_key_hex: format!("0x{}", hex::encode(pk.serialize())),
        address: pubkey_to_address(&pk),
    }
}

fn wallet_from_privkey(privkey_hex: &str) -> Wallet {
    let bytes =
        hex::decode(privkey_hex.strip_prefix("0x").unwrap_or(privkey_hex)).expect("bad hex");
    let arr: [u8; 32] = bytes.try_into().expect("privkey must be 32 bytes");
    let sk = SecretKey::parse(&arr).expect("invalid privkey");
    let pk = PublicKey::from_secret_key(&sk);
    Wallet {
        secret_key: sk,
        public_key_hex: format!("0x{}", hex::encode(pk.serialize())),
        address: pubkey_to_address(&pk),
    }
}

fn cli_wallet() {
    let w = create_wallet();
    println!("Address:    {}", w.address);
    println!("PublicKey:  {}", w.public_key_hex);
    println!("PrivateKey: 0x{}", hex::encode(w.secret_key.serialize()));
}

async fn cli_send(args: &[String]) {
    if args.len() != 4 {
        eprintln!("Usage: send <node_url> <to> <amount> <private_key>");
        return;
    }
    let node_url = args[0].trim_end_matches('/');
    let to = args[1].to_lowercase();
    let amount: u64 = args[2].parse().expect("bad amount");
    let w = wallet_from_privkey(&args[3]);
    let ts = now_ms();

    let canonical = format!("TRANSFER|{}|{}|{}|{}", w.address, to, amount, ts);
    let sig = sign_message(&canonical, &w.secret_key);

    let tx = Transaction {
        id: Uuid::new_v4().to_string(),
        tx_type: "TRANSFER".into(),
        from: w.address.clone(),
        to: to.clone(),
        amount,
        timestamp: ts,
        public_key: w.public_key_hex.clone(),
        signature: sig,
    };

    println!("Sending {} from {} to {}", amount, w.address, to);
    let client = reqwest::Client::new();
    match client
        .post(format!("{node_url}/transactions"))
        .json(&tx)
        .send()
        .await
    {
        Ok(resp) => {
            let status = resp.status();
            let body: serde_json::Value = resp.json().await.unwrap_or(json!({}));
            println!("HTTP {status}: {body}");
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}

async fn cli_balance(args: &[String]) {
    if args.len() != 2 {
        eprintln!("Usage: balance <node_url> <address>");
        return;
    }
    let node_url = args[0].trim_end_matches('/');
    let address = &args[1];
    let client = reqwest::Client::new();
    match client
        .get(format!("{node_url}/balance/{address}"))
        .send()
        .await
    {
        Ok(resp) => {
            let body: serde_json::Value = resp.json().await.unwrap_or(json!({}));
            println!("{body}");
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}

async fn cli_mine(args: &[String]) {
    if args.len() != 1 {
        eprintln!("Usage: mine <node_url>");
        return;
    }
    let node_url = args[0].trim_end_matches('/');
    let client = reqwest::Client::new();
    match client
        .post(format!("{node_url}/mine"))
        .json(&json!({}))
        .send()
        .await
    {
        Ok(resp) => {
            let body: serde_json::Value = resp.json().await.unwrap_or(json!({}));
            println!("{body}");
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}

async fn cli_peers(args: &[String]) {
    if args.len() != 1 {
        eprintln!("Usage: peers <node_url>");
        return;
    }
    let node_url = args[0].trim_end_matches('/');
    let client = reqwest::Client::new();
    match client.get(format!("{node_url}/peers")).send().await {
        Ok(resp) => {
            let body: serde_json::Value = resp.json().await.unwrap_or(json!({}));
            println!("{body}");
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}

async fn cli_chain(args: &[String]) {
    if args.len() != 1 {
        eprintln!("Usage: chain <node_url>");
        return;
    }
    let node_url = args[0].trim_end_matches('/');
    let client = reqwest::Client::new();
    match client.get(format!("{node_url}/status")).send().await {
        Ok(resp) => {
            let body: serde_json::Value = resp.json().await.unwrap_or(json!({}));
            println!("{body}");
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}

async fn run_node(port: u16) {
    let node_url =
        std::env::var("NODE_URL").unwrap_or_else(|_| format!("http://127.0.0.1:{port}"));

    let wallet = create_wallet();
    println!("Address:   {}", wallet.address);
    println!("PublicKey: {}", wallet.public_key_hex);

    println!("Mining genesis block...");
    let genesis = create_genesis();
    println!("Genesis: {} (nonce={})", genesis.hash, genesis.nonce);

    let state = Arc::new(Mutex::new(AppState {
        chain: vec![genesis],
        mempool: vec![],
        peers: HashSet::new(),
        wallet,
        node_url: node_url.clone(),
    }));

    if let Ok(seeds) = std::env::var("SEED_PEERS") {
        let seed_list: Vec<String> = seeds.split(',').map(|s| s.trim().to_string()).collect();
        bootstrap(&state, &seed_list).await;
    }

    let app = Router::new()
        .route("/health", get(routes::health))
        .route("/status", get(routes::get_status))
        .route("/chain", get(routes::get_chain))
        .route("/mempool", get(routes::get_mempool))
        .route("/balance/{addr}", get(routes::get_balance_route))
        .route("/transactions", post(routes::post_transactions))
        .route("/mine", post(routes::post_mine_route))
        .route("/blocks", post(routes::post_blocks))
        .route("/peers", get(routes::get_peers).post(routes::post_peers))
        .with_state(state);

    println!("Node running at {node_url}");
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("wallet")  => cli_wallet(),
        Some("send")    => cli_send(&args[2..]).await,
        Some("balance") => cli_balance(&args[2..]).await,
        Some("mine")    => cli_mine(&args[2..]).await,
        Some("peers")   => cli_peers(&args[2..]).await,
        Some("chain")   => cli_chain(&args[2..]).await,
        Some(port_str)  => {
            let port: u16 = port_str.parse().expect("invalid port number");
            run_node(port).await;
        }
        None => {
            eprintln!("Usage:");
            eprintln!("  blockchain-p2p <port>                                     Run node");
            eprintln!("  blockchain-p2p wallet                                     Generate wallet");
            eprintln!("  blockchain-p2p send    <node_url> <to> <amount> <privkey> Send transaction");
            eprintln!("  blockchain-p2p balance <node_url> <address>               Check balance");
            eprintln!("  blockchain-p2p mine    <node_url>                         Trigger mining");
            eprintln!("  blockchain-p2p peers   <node_url>                         List peers");
            eprintln!("  blockchain-p2p chain   <node_url>                         Chain status");
        }
    }
}
