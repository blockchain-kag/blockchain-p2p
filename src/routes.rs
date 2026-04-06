use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use std::collections::HashSet;

use crate::mining::mine_block;
use crate::network::{broadcast_block_to, broadcast_tx_to, resolve_conflicts};
use crate::types::{Block, Shared, Transaction, AUTO_MINE_THRESHOLD};
use crate::validation::{get_balance, validate_block, validate_transfer};

pub async fn health() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

pub async fn get_status(State(state): State<Shared>) -> Json<Value> {
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

pub async fn get_chain(State(state): State<Shared>) -> Json<Value> {
    let s = state.lock().unwrap();
    Json(json!({
        "status": "ok",
        "chain": s.chain,
        "length": s.chain.len()
    }))
}

pub async fn get_mempool(State(state): State<Shared>) -> Json<Value> {
    let s = state.lock().unwrap();
    Json(json!({
        "status": "ok",
        "transactions": s.mempool,
        "count": s.mempool.len()
    }))
}

pub async fn get_balance_route(
    State(state): State<Shared>,
    Path(addr): Path<String>,
) -> Json<Value> {
    let s = state.lock().unwrap();
    let bal = get_balance(&addr, &s.chain);
    Json(json!({"status": "ok", "address": addr, "balance": bal}))
}

pub async fn post_transactions(
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

        // Dedup
        if s.mempool.iter().any(|t| t.id == tx.id) {
            return (
                StatusCode::ACCEPTED,
                Json(json!({"status": "ok", "accepted": true, "txId": tx.id})),
            );
        }
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

    let tx2 = tx.clone();
    let peers2 = peers.clone();
    tokio::spawn(async move {
        broadcast_tx_to(&tx2, &peers2).await;
    });

    if should_mine {
        let st = state.clone();
        tokio::spawn(async move {
            let (block, peers) = {
                let mut s = st.lock().unwrap();
                let blk = mine_block(&mut s);
                let p: Vec<String> = s.peers.iter().cloned().collect();
                (blk, p)
            };
            println!("[auto-mine] block #{} hash={}", block.index, block.hash);
            broadcast_block_to(&block, &peers).await;
        });
    }

    (
        StatusCode::ACCEPTED,
        Json(json!({"status": "ok", "accepted": true, "txId": tx.id})),
    )
}

pub async fn post_mine_route(State(state): State<Shared>) -> Json<Value> {
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

pub async fn post_blocks(
    State(state): State<Shared>,
    Json(blk): Json<Block>,
) -> (StatusCode, Json<Value>) {
    let action = {
        let mut s = state.lock().unwrap();

        // Clone what we need from `last` to avoid holding a borrow on s.chain
        let (last_index, last_hash) = {
            let last = s.chain.last().unwrap();
            (last.index, last.hash.clone())
        };

        if blk.index == last_index + 1 && blk.previous_hash == last_hash {
            // Re-borrow last for validation (borrow ends when validate_block returns)
            let prev = s.chain.last().cloned().unwrap();
            match validate_block(&blk, Some(&prev), &s.chain) {
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
                Err(e) => return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "status": "error",
                        "error": {"code": "INVALID_BLOCK", "message": e}
                    })),
                ),
            }
        } else if blk.index > last_index + 1 {
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
            (StatusCode::OK, Json(json!({"status": "ok", "accepted": true, "action": "appended", "chainLength": len})))
        }
        Some(("syncing", _, _)) => {
            let st = state.clone();
            tokio::spawn(async move {
                resolve_conflicts(&st).await;
            });
            let len = state.lock().unwrap().chain.len();
            (StatusCode::OK, Json(json!({"status": "ok", "accepted": true, "action": "syncing", "chainLength": len})))
        }
        Some((act, accepted, len)) => {
            (StatusCode::OK, Json(json!({"status": "ok", "accepted": accepted, "action": act, "chainLength": len})))
        }
        None => unreachable!(),
    }
}

pub async fn get_peers(State(state): State<Shared>) -> Json<Value> {
    let s = state.lock().unwrap();
    let list: Vec<&String> = s.peers.iter().collect();
    Json(json!({
        "status": "ok",
        "peers": list,
        "count": s.peers.len()
    }))
}

pub async fn post_peers(State(state): State<Shared>, Json(body): Json<Value>) -> Json<Value> {
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
    // Return peers excluding the one that just registered (they don't need themselves)
    let list: Vec<String> = s
        .peers
        .iter()
        .filter(|p| p.as_str() != url.as_str())
        .cloned()
        .collect();
    Json(json!({
        "status": "ok",
        "registered": url,
        "peers": list
    }))
}
