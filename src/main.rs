// Blockchain P2P Node
// Usage:
//   cargo run -- <port>                                    Run node
//   cargo run -- wallet                                    Generate wallet
//   cargo run -- send <node_url> <to> <amount> <privkey>   Send transaction
//
// Env:
//   SEED_PEERS=http://ip:port,http://ip:port
//   NODE_URL=http://my_ip:port

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use libsecp256k1::{recover, sign, Message, PublicKey, RecoveryId, SecretKey};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sha3::Keccak256;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

// ── Constants ──────────────────────────────────────────────

const DIFFICULTY: usize = 4;
const BLOCK_REWARD: u64 = 10;
const AUTO_MINE_THRESHOLD: usize = 3;
const GENESIS_TIMESTAMP: u64 = 1743086300000;
const ZEROS: &str = "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

// ── Data Structures ───────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    #[serde(rename = "type")]
    pub tx_type: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub timestamp: u64,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    #[serde(rename = "previousHash")]
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
}

struct Wallet {
    secret_key: SecretKey,
    public_key_hex: String,
    address: String,
}

struct AppState {
    chain: Vec<Block>,
    mempool: Vec<Transaction>,
    peers: HashSet<String>,
    wallet: Wallet,
    node_url: String,
}

type Shared = Arc<Mutex<AppState>>;

// ── Helpers ───────────────────────────────────────────────

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn difficulty_prefix() -> String {
    "0".repeat(DIFFICULTY)
}

// ── Crypto ────────────────────────────────────────────────

fn pubkey_to_address(pk: &PublicKey) -> String {
    let raw = pk.serialize();
    let hash = Keccak256::digest(&raw[1..]);
    format!("0x{}", hex::encode(&hash[12..]))
}

fn address_from_pubkey_hex(pubkey_hex: &str) -> Result<String, String> {
    let raw = hex::decode(pubkey_hex.strip_prefix("0x").unwrap_or(pubkey_hex))
        .map_err(|e| format!("hex: {e}"))?;
    let pk = match raw.len() {
        65 => {
            let arr: [u8; 65] = raw.try_into().unwrap();
            PublicKey::parse(&arr).map_err(|e| format!("pk: {e}"))?
        }
        33 => {
            let arr: [u8; 33] = raw.try_into().unwrap();
            PublicKey::parse_compressed(&arr).map_err(|e| format!("pk: {e}"))?
        }
        n => return Err(format!("bad pubkey len: {n}")),
    };
    Ok(pubkey_to_address(&pk))
}

fn eth_message_hash(message: &str) -> [u8; 32] {
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    let mut h = Keccak256::new();
    h.update(prefix.as_bytes());
    h.update(message.as_bytes());
    h.finalize().into()
}

fn sign_message(message: &str, sk: &SecretKey) -> String {
    let hash = eth_message_hash(message);
    let msg = Message::parse(&hash);
    let (sig, recid) = sign(&msg, sk);
    let mut out = [0u8; 65];
    out[..64].copy_from_slice(&sig.serialize());
    out[64] = recid.serialize() + 27;
    format!("0x{}", hex::encode(out))
}

fn recover_address(message: &str, sig_hex: &str) -> Result<String, String> {
    let bytes = hex::decode(sig_hex.strip_prefix("0x").unwrap_or(sig_hex))
        .map_err(|e| format!("sig hex: {e}"))?;
    if bytes.len() != 65 {
        return Err(format!("sig len {}", bytes.len()));
    }
    let arr: [u8; 64] = bytes[..64].try_into().unwrap();
    let sig =
        libsecp256k1::Signature::parse_standard(&arr).map_err(|e| format!("parse sig: {e}"))?;
    let v = bytes[64];
    let recid_byte = if v >= 27 { v - 27 } else { v };
    let recid = RecoveryId::parse(recid_byte).map_err(|e| format!("recid: {e}"))?;
    let hash = eth_message_hash(message);
    let msg = Message::parse(&hash);
    let recovered = recover(&msg, &sig, &recid).map_err(|e| format!("recover: {e}"))?;
    Ok(pubkey_to_address(&recovered))
}

// ── Block Hashing ─────────────────────────────────────────

fn compute_block_hash(
    index: u64,
    timestamp: u64,
    prev_hash: &str,
    nonce: u64,
    txs: &[Transaction],
) -> String {
    let tx_ids: Vec<&str> = txs.iter().map(|t| t.id.as_str()).collect();
    let data = format!(
        "{}|{}|{}|{}|{}",
        index,
        timestamp,
        prev_hash,
        nonce,
        tx_ids.join(",")
    );
    hex::encode(Sha256::digest(data.as_bytes()))
}

// ── Genesis ───────────────────────────────────────────────

fn create_genesis() -> Block {
    let prefix = difficulty_prefix();
    let mut nonce = 0u64;
    loop {
        let hash = compute_block_hash(0, GENESIS_TIMESTAMP, "0", nonce, &[]);
        if hash.starts_with(&prefix) {
            return Block {
                index: 0,
                timestamp: GENESIS_TIMESTAMP,
                transactions: vec![],
                previous_hash: "0".into(),
                hash,
                nonce,
            };
        }
        nonce += 1;
    }
}

// ── Balance ───────────────────────────────────────────────

fn get_balance(addr: &str, chain: &[Block]) -> i64 {
    let a = addr.to_lowercase();
    let mut bal: i64 = 0;
    for blk in chain {
        for tx in &blk.transactions {
            if tx.to.to_lowercase() == a {
                bal += tx.amount as i64;
            }
            if tx.from != "SYSTEM" && tx.from.to_lowercase() == a {
                bal -= tx.amount as i64;
            }
        }
    }
    bal
}

// ── Validation ────────────────────────────────────────────

fn validate_coinbase(tx: &Transaction, block_ts: u64) -> Result<(), String> {
    if tx.tx_type != "COINBASE" {
        return Err("not COINBASE".into());
    }
    if tx.from != "SYSTEM" {
        return Err("from != SYSTEM".into());
    }
    if tx.amount != BLOCK_REWARD {
        return Err(format!("reward {} != {BLOCK_REWARD}", tx.amount));
    }
    if tx.timestamp != block_ts {
        return Err("coinbase ts mismatch".into());
    }
    let pk = tx.public_key.replace("0x", "");
    if pk.is_empty() || !pk.chars().all(|c| c == '0') {
        return Err("coinbase pk not zeros".into());
    }
    let sg = tx.signature.replace("0x", "");
    if sg.is_empty() || !sg.chars().all(|c| c == '0') {
        return Err("coinbase sig not zeros".into());
    }
    Ok(())
}

fn validate_transfer(tx: &Transaction, chain: &[Block]) -> Result<(), String> {
    if tx.tx_type != "TRANSFER" {
        return Err("not TRANSFER".into());
    }
    if tx.id.is_empty() {
        return Err("missing id".into());
    }
    if tx.from.is_empty() || tx.to.is_empty() {
        return Err("missing from/to".into());
    }
    if tx.from == tx.to {
        return Err("from == to".into());
    }
    if tx.amount == 0 {
        return Err("amount == 0".into());
    }
    if tx.timestamp == 0 {
        return Err("timestamp == 0".into());
    }
    if tx.public_key.is_empty() {
        return Err("missing publicKey".into());
    }
    if tx.signature.is_empty() {
        return Err("missing signature".into());
    }

    // Verify signature
    let canonical = format!(
        "TRANSFER|{}|{}|{}|{}",
        tx.from, tx.to, tx.amount, tx.timestamp
    );
    let recovered = recover_address(&canonical, &tx.signature)?;
    if recovered.to_lowercase() != tx.from.to_lowercase() {
        return Err(format!(
            "sig mismatch: recovered={recovered} from={}",
            tx.from
        ));
    }

    // Verify publicKey derives to from
    let derived = address_from_pubkey_hex(&tx.public_key)?;
    if derived.to_lowercase() != tx.from.to_lowercase() {
        return Err("publicKey doesn't match from".into());
    }

    // Balance
    let bal = get_balance(&tx.from, chain);
    if bal < tx.amount as i64 {
        return Err(format!("insufficient balance: {bal} < {}", tx.amount));
    }

    Ok(())
}

fn validate_block(blk: &Block, prev: Option<&Block>, chain_before: &[Block]) -> Result<(), String> {
    if blk.timestamp == 0 {
        return Err("ts == 0".into());
    }

    // Recompute hash
    let computed = compute_block_hash(
        blk.index,
        blk.timestamp,
        &blk.previous_hash,
        blk.nonce,
        &blk.transactions,
    );
    if computed != blk.hash {
        return Err(format!("hash mismatch: {computed} != {}", blk.hash));
    }
    if !blk.hash.starts_with(&difficulty_prefix()) {
        return Err("PoW fail".into());
    }

    // Genesis
    if blk.index == 0 {
        if blk.previous_hash != "0" {
            return Err("genesis prevHash".into());
        }
        if !blk.transactions.is_empty() {
            return Err("genesis has txs".into());
        }
        return Ok(());
    }

    // Non-genesis
    let prev = prev.ok_or("no previous block")?;
    if blk.previous_hash != prev.hash {
        return Err("prevHash mismatch".into());
    }
    if blk.index != prev.index + 1 {
        return Err("index gap".into());
    }
    if blk.timestamp <= prev.timestamp {
        return Err("ts not increasing".into());
    }
    if blk.transactions.is_empty() {
        return Err("no txs".into());
    }

    // COINBASE checks
    validate_coinbase(&blk.transactions[0], blk.timestamp)?;
    let cb_count = blk
        .transactions
        .iter()
        .filter(|t| t.tx_type == "COINBASE")
        .count();
    if cb_count != 1 {
        return Err(format!("{cb_count} coinbases"));
    }

    // TRANSFER checks
    for tx in &blk.transactions[1..] {
        validate_transfer(tx, chain_before)?;
    }

    Ok(())
}

fn validate_full_chain(chain: &[Block]) -> bool {
    if chain.is_empty() {
        return false;
    }
    let mut temp: Vec<Block> = Vec::new();
    for (i, blk) in chain.iter().enumerate() {
        let prev = if i > 0 { Some(&chain[i - 1]) } else { None };
        if validate_block(blk, prev, &temp).is_err() {
            return false;
        }
        temp.push(blk.clone());
    }
    true
}

// ── Mining ────────────────────────────────────────────────

fn mine_block(state: &mut AppState) -> Block {
    let prev = state.chain.last().unwrap();
    let ts = now_ms();
    let coinbase = Transaction {
        id: Uuid::new_v4().to_string(),
        tx_type: "COINBASE".into(),
        from: "SYSTEM".into(),
        to: state.wallet.address.clone(),
        amount: BLOCK_REWARD,
        timestamp: ts,
        public_key: ZEROS.into(),
        signature: ZEROS.into(),
    };

    let mut txs = vec![coinbase];
    txs.extend(state.mempool.drain(..));

    let index = prev.index + 1;
    let prev_hash = prev.hash.clone();
    let prefix = difficulty_prefix();
    let mut nonce = 0u64;

    loop {
        let hash = compute_block_hash(index, ts, &prev_hash, nonce, &txs);
        if hash.starts_with(&prefix) {
            let block = Block {
                index,
                timestamp: ts,
                transactions: txs,
                previous_hash: prev_hash,
                hash,
                nonce,
            };
            state.chain.push(block.clone());
            return block;
        }
        nonce += 1;
    }
}

// ── Network ───────────────────────────────────────────────

async fn broadcast_block_to(block: &Block, peers: &[String]) {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();
    for peer in peers {
        let _ = client
            .post(format!("{peer}/blocks"))
            .json(block)
            .send()
            .await;
    }
}

async fn broadcast_tx_to(tx: &Transaction, peers: &[String]) {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();
    for peer in peers {
        let _ = client
            .post(format!("{peer}/transactions"))
            .json(tx)
            .send()
            .await;
    }
}

async fn resolve_conflicts(state: &Shared) {
    let peers: Vec<String> = state.lock().unwrap().peers.iter().cloned().collect();
    let current_len = state.lock().unwrap().chain.len();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();
    let mut best_chain: Option<Vec<Block>> = None;
    let mut best_len = current_len;

    for peer in &peers {
        let resp = match client.get(format!("{peer}/chain")).send().await {
            Ok(r) => r,
            Err(_) => continue,
        };
        let data: Value = match resp.json().await {
            Ok(d) => d,
            Err(_) => continue,
        };
        if let Some(remote) = data.get("chain") {
            if let Ok(rc) = serde_json::from_value::<Vec<Block>>(remote.clone()) {
                if rc.len() > best_len && validate_full_chain(&rc) {
                    best_len = rc.len();
                    best_chain = Some(rc);
                }
            }
        }
    }

    if let Some(new_chain) = best_chain {
        let mut s = state.lock().unwrap();
        if new_chain.len() > s.chain.len() {
            s.chain = new_chain;
        }
    }
}

async fn bootstrap(state: &Shared, seeds: &[String]) {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();
    let node_url = state.lock().unwrap().node_url.clone();

    for seed in seeds {
        let seed = seed.trim().trim_end_matches('/');
        if seed.is_empty() {
            continue;
        }

        // 1. Contact seed
        if client.get(format!("{seed}/status")).send().await.is_err() {
            println!("[bootstrap] FAIL {seed}: unreachable");
            continue;
        }

        // 2. Download chain
        if let Ok(resp) = client.get(format!("{seed}/chain")).send().await {
            if let Ok(data) = resp.json::<Value>().await {
                if let Some(remote) = data.get("chain") {
                    if let Ok(rc) = serde_json::from_value::<Vec<Block>>(remote.clone()) {
                        let mut s = state.lock().unwrap();
                        if rc.len() > s.chain.len() && validate_full_chain(&rc) {
                            s.chain = rc;
                        }
                    }
                }
            }
        }

        // 3. Register ourselves & get peers
        if let Ok(resp) = client
            .post(format!("{seed}/peers"))
            .json(&json!({"url": node_url}))
            .send()
            .await
        {
            if let Ok(data) = resp.json::<Value>().await {
                let mut s = state.lock().unwrap();
                if let Some(arr) = data.get("peers").and_then(|v| v.as_array()) {
                    for p in arr {
                        if let Some(url) = p.as_str() {
                            let url = url.trim_end_matches('/').to_string();
                            if url != node_url {
                                s.peers.insert(url);
                            }
                        }
                    }
                }
                s.peers.insert(seed.to_string());
            }
        }

        println!("[bootstrap] OK from {seed}");
        break;
    }
}

// ── API Routes ────────────────────────────────────────────

async fn health() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

async fn get_status(State(state): State<Shared>) -> Json<Value> {
    let s = state.lock().unwrap();
    Json(json!({
        "status": "ok",
        "node": {
            "url": s.node_url,
            "address": s.wallet.address,
            "publicKey": s.wallet.public_key_hex
        },
        "chain": {
            "length": s.chain.len(),
            "latestHash": s.chain.last().map(|b| b.hash.as_str()).unwrap_or("")
        },
        "peers": {
            "count": s.peers.len()
        }
    }))
}

async fn get_chain(State(state): State<Shared>) -> Json<Value> {
    let s = state.lock().unwrap();
    Json(json!({
        "status": "ok",
        "chain": s.chain,
        "length": s.chain.len()
    }))
}

async fn get_mempool(State(state): State<Shared>) -> Json<Value> {
    let s = state.lock().unwrap();
    Json(json!({
        "status": "ok",
        "transactions": s.mempool,
        "count": s.mempool.len()
    }))
}

async fn get_balance_route(
    State(state): State<Shared>,
    Path(addr): Path<String>,
) -> Json<Value> {
    let s = state.lock().unwrap();
    let bal = get_balance(&addr, &s.chain);
    Json(json!({"status": "ok", "address": addr, "balance": bal}))
}

async fn post_transactions(
    State(state): State<Shared>,
    Json(tx): Json<Transaction>,
) -> (StatusCode, Json<Value>) {
    if tx.tx_type == "COINBASE" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "error": {"code": "INVALID_TRANSACTION", "message": "COINBASE rejected"}
            })),
        );
    }

    let (peers, should_mine) = {
        let mut s = state.lock().unwrap();

        if let Err(e) = validate_transfer(&tx, &s.chain) {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "error",
                    "error": {"code": "INVALID_TRANSACTION", "message": e}
                })),
            );
        }

        // Dedup: already in mempool?
        if s.mempool.iter().any(|t| t.id == tx.id) {
            return (
                StatusCode::ACCEPTED,
                Json(json!({"status": "ok", "accepted": true, "txId": tx.id})),
            );
        }
        // Dedup: already in chain?
        for blk in &s.chain {
            if blk.transactions.iter().any(|t| t.id == tx.id) {
                return (
                    StatusCode::ACCEPTED,
                    Json(json!({"status": "ok", "accepted": true, "txId": tx.id})),
                );
            }
        }

        s.mempool.push(tx.clone());
        let count = s.mempool.len();
        let peers: Vec<String> = s.peers.iter().cloned().collect();
        (peers, count >= AUTO_MINE_THRESHOLD)
    };

    // Broadcast to peers
    let tx2 = tx.clone();
    let peers2 = peers.clone();
    tokio::spawn(async move {
        broadcast_tx_to(&tx2, &peers2).await;
    });

    // Auto-mine
    if should_mine {
        let st = state.clone();
        tokio::spawn(async move {
            let (block, peers) = {
                let mut s = st.lock().unwrap();
                let blk = mine_block(&mut s);
                let p: Vec<String> = s.peers.iter().cloned().collect();
                (blk, p)
            };
            println!(
                "[auto-mine] block #{} hash={}",
                block.index, block.hash
            );
            broadcast_block_to(&block, &peers).await;
        });
    }

    (
        StatusCode::ACCEPTED,
        Json(json!({"status": "ok", "accepted": true, "txId": tx.id})),
    )
}

async fn post_mine_route(State(state): State<Shared>) -> Json<Value> {
    let (block, peers) = {
        let mut s = state.lock().unwrap();
        let blk = mine_block(&mut s);
        let p: Vec<String> = s.peers.iter().cloned().collect();
        (blk, p)
    };
    let block2 = block.clone();
    tokio::spawn(async move {
        broadcast_block_to(&block2, &peers).await;
    });
    Json(json!({
        "status": "ok",
        "mined": true,
        "trigger": "manual",
        "block": block
    }))
}

async fn post_blocks(State(state): State<Shared>, Json(blk): Json<Block>) -> Json<Value> {
    let action = {
        let mut s = state.lock().unwrap();
        let last = s.chain.last().unwrap();

        if blk.index == last.index + 1 && blk.previous_hash == last.hash {
            match validate_block(&blk, Some(last), &s.chain) {
                Ok(()) => {
                    let mined: HashSet<String> = blk
                        .transactions
                        .iter()
                        .filter(|t| t.tx_type == "TRANSFER")
                        .map(|t| t.id.clone())
                        .collect();
                    s.mempool.retain(|t| !mined.contains(&t.id));
                    s.chain.push(blk.clone());
                    let len = s.chain.len();
                    Some(("appended", true, len))
                }
                Err(_) => None,
            }
        } else if blk.index > last.index + 1 {
            Some(("syncing", true, s.chain.len()))
        } else {
            Some(("ignored", false, s.chain.len()))
        }
    };

    match action {
        Some(("appended", _, len)) => {
            let peers: Vec<String> = state.lock().unwrap().peers.iter().cloned().collect();
            let blk2 = blk.clone();
            tokio::spawn(async move {
                broadcast_block_to(&blk2, &peers).await;
            });
            Json(json!({"status": "ok", "accepted": true, "action": "appended", "chainLength": len}))
        }
        Some(("syncing", _, _)) => {
            let st = state.clone();
            tokio::spawn(async move {
                resolve_conflicts(&st).await;
            });
            let len = state.lock().unwrap().chain.len();
            Json(json!({"status": "ok", "accepted": true, "action": "syncing", "chainLength": len}))
        }
        Some((act, accepted, len)) => {
            Json(json!({"status": "ok", "accepted": accepted, "action": act, "chainLength": len}))
        }
        None => Json(json!({
            "status": "error",
            "error": {"code": "INVALID_BLOCK", "message": "validation failed"}
        })),
    }
}

async fn get_peers(State(state): State<Shared>) -> Json<Value> {
    let s = state.lock().unwrap();
    let list: Vec<&String> = s.peers.iter().collect();
    Json(json!({
        "status": "ok",
        "peers": list,
        "count": s.peers.len()
    }))
}

async fn post_peers(State(state): State<Shared>, Json(body): Json<Value>) -> Json<Value> {
    let url = body
        .get("url")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim_end_matches('/')
        .to_string();

    let mut s = state.lock().unwrap();
    if !url.is_empty() && url != s.node_url {
        s.peers.insert(url.clone());
    }
    let list: Vec<String> = s.peers.iter().cloned().collect();
    Json(json!({
        "status": "ok",
        "registered": url,
        "peers": list
    }))
}

// ── Wallet helpers ────────────────────────────────────────

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

// ── CLI ───────────────────────────────────────────────────

fn cli_wallet() {
    let w = create_wallet();
    println!("Address:    {}", w.address);
    println!("PublicKey:  {}", w.public_key_hex);
    println!(
        "PrivateKey: 0x{}",
        hex::encode(w.secret_key.serialize())
    );
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
            let body: Value = resp.json().await.unwrap_or(json!({}));
            println!("HTTP {status}: {body}");
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}

// ── Main ──────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("wallet") => {
            cli_wallet();
        }
        Some("send") => {
            cli_send(&args[2..]).await;
        }
        Some(port_str) => {
            let port: u16 = port_str.parse().expect("invalid port number");
            run_node(port).await;
        }
        None => {
            eprintln!("Usage:");
            eprintln!("  blockchain-p2p <port>                                  Run node");
            eprintln!("  blockchain-p2p wallet                                  Generate wallet");
            eprintln!("  blockchain-p2p send <node_url> <to> <amount> <privkey> Send tx");
        }
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

    // Bootstrap from seed peers
    if let Ok(seeds) = std::env::var("SEED_PEERS") {
        let seed_list: Vec<String> = seeds.split(',').map(|s| s.trim().to_string()).collect();
        bootstrap(&state, &seed_list).await;
    }

    let app = Router::new()
        .route("/health", get(health))
        .route("/status", get(get_status))
        .route("/chain", get(get_chain))
        .route("/mempool", get(get_mempool))
        .route("/balance/{addr}", get(get_balance_route))
        .route("/transactions", post(post_transactions))
        .route("/mine", post(post_mine_route))
        .route("/blocks", post(post_blocks))
        .route("/peers", get(get_peers).post(post_peers))
        .with_state(state);

    println!("Node running at {node_url}");
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
