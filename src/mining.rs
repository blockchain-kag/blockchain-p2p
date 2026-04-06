use uuid::Uuid;

use crate::crypto::{compute_block_hash, now_ms};
use crate::types::{AppState, Block, Transaction, BLOCK_REWARD, DIFFICULTY, GENESIS_TIMESTAMP, ZEROS};

fn difficulty_prefix() -> String {
    "0".repeat(DIFFICULTY)
}

pub fn create_genesis() -> Block {
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

pub fn mine_block(state: &mut AppState) -> Block {
    // Extract what we need from prev before any other mutable borrows
    let (index, prev_hash) = {
        let prev = state.chain.last().unwrap();
        (prev.index + 1, prev.hash.clone())
    };
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
