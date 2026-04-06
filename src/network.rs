use serde_json::{json, Value};

use crate::types::{Block, Shared, Transaction};
use crate::validation::validate_full_chain;

pub async fn broadcast_block_to(block: &Block, peers: &[String]) {
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

pub async fn broadcast_tx_to(tx: &Transaction, peers: &[String]) {
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

pub async fn resolve_conflicts(state: &Shared) {
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

pub async fn bootstrap(state: &Shared, seeds: &[String]) {
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

        if client.get(format!("{seed}/status")).send().await.is_err() {
            println!("[bootstrap] FAIL {seed}: unreachable");
            continue;
        }

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

        // 3. Register ourselves with seed, get its peer list
        let discovered_peers: Vec<String> = if let Ok(resp) = client
            .post(format!("{seed}/peers"))
            .json(&json!({"url": node_url}))
            .send()
            .await
        {
            if let Ok(data) = resp.json::<Value>().await {
                let peers: Vec<String> = data
                    .get("peers")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|p| p.as_str())
                            .map(|u| u.trim_end_matches('/').to_string())
                            .filter(|u| u != &node_url)
                            .collect()
                    })
                    .unwrap_or_default();

                let mut s = state.lock().unwrap();
                for p in &peers {
                    s.peers.insert(p.clone());
                }
                s.peers.insert(seed.to_string());
                peers
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        // 4. Also register ourselves with every peer we discovered
        //    so they know we exist and will send us blocks
        for peer in &discovered_peers {
            let _ = client
                .post(format!("{peer}/peers"))
                .json(&json!({"url": node_url}))
                .send()
                .await;
        }

        println!("[bootstrap] OK from {seed}, discovered {} peers", discovered_peers.len());
        break;
    }
}
