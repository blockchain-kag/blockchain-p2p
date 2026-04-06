use crate::crypto::{address_from_pubkey_hex, compute_block_hash, recover_address};
use crate::types::{Block, Transaction, BLOCK_REWARD, DIFFICULTY};

fn difficulty_prefix() -> String {
    "0".repeat(DIFFICULTY)
}

pub fn get_balance(addr: &str, chain: &[Block]) -> i64 {
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

pub fn validate_coinbase(tx: &Transaction, block_ts: u64) -> Result<(), String> {
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

pub fn validate_transfer(tx: &Transaction, chain: &[Block]) -> Result<(), String> {
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

pub fn validate_block(
    blk: &Block,
    prev: Option<&Block>,
    chain_before: &[Block],
) -> Result<(), String> {
    if blk.timestamp == 0 {
        return Err("ts == 0".into());
    }

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

    // COINBASE
    validate_coinbase(&blk.transactions[0], blk.timestamp)?;
    let cb_count = blk
        .transactions
        .iter()
        .filter(|t| t.tx_type == "COINBASE")
        .count();
    if cb_count != 1 {
        return Err(format!("{cb_count} coinbases"));
    }

    // TRANSFERs
    for tx in &blk.transactions[1..] {
        validate_transfer(tx, chain_before)?;
    }

    Ok(())
}

pub fn validate_full_chain(chain: &[Block]) -> bool {
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
