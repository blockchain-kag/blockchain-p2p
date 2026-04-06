use libsecp256k1::{recover, sign, Message, PublicKey, RecoveryId, SecretKey};
use sha2::{Digest, Sha256};
use sha3::Keccak256;

use crate::types::Transaction;

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub fn pubkey_to_address(pk: &PublicKey) -> String {
    let raw = pk.serialize();
    let hash = Keccak256::digest(&raw[1..]);
    format!("0x{}", hex::encode(&hash[12..]))
}

pub fn address_from_pubkey_hex(pubkey_hex: &str) -> Result<String, String> {
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

pub fn sign_message(message: &str, sk: &SecretKey) -> String {
    let hash = eth_message_hash(message);
    let msg = Message::parse(&hash);
    let (sig, recid) = sign(&msg, sk);
    let mut out = [0u8; 65];
    out[..64].copy_from_slice(&sig.serialize());
    out[64] = recid.serialize() + 27;
    format!("0x{}", hex::encode(out))
}

pub fn recover_address(message: &str, sig_hex: &str) -> Result<String, String> {
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

pub fn compute_block_hash(
    index: u64,
    timestamp: u64,
    prev_hash: &str,
    nonce: u64,
    txs: &[Transaction],
) -> String {
    let tx_ids: Vec<&str> = txs.iter().map(|t| t.id.as_str()).collect();
    let data = format!(
        "{}|{}|{}|{}|{}",
        index, timestamp, prev_hash, nonce,
        tx_ids.join(",")
    );
    hex::encode(Sha256::digest(data.as_bytes()))
}
