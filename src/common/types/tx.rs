use crate::{common::ports::hasher::Hasher, common::ports::signing_key::SigningKey};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    pub fn zero() -> Hash {
        Hash([0; 32])
    }
}

#[derive(Serialize, Deserialize)]
struct TxData {
    prev_tx_hash: Hash,
    sender: Vec<u8>,
    recipient: Vec<u8>,
    amount: u64,
}

impl TxData {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.prev_tx_hash.0);
        bytes.extend_from_slice(&self.sender);
        bytes.extend_from_slice(&self.recipient);
        bytes.extend_from_slice(&self.amount.to_be_bytes());

        bytes
    }
}

#[derive(Serialize, Deserialize)]
pub struct Tx {
    data: TxData,
    signature: Vec<u8>,
}

impl Tx {
    pub fn new_signed<SK: SigningKey>(
        prev_tx_hash: Hash,
        from: Vec<u8>,
        to: Vec<u8>,
        amount: u64,
        sk: &SK,
        hasher: &dyn Hasher,
    ) -> Self {
        let data = TxData {
            prev_tx_hash,
            recipient: to,
            sender: from,
            amount,
        };
        let msg = Self::msg(hasher, &data);
        let signature = sk.sign(&msg.0);

        Self { data, signature }
    }

    fn msg(hasher: &dyn Hasher, data: &TxData) -> Hash {
        let bytes = data.to_bytes();
        hasher.hash(&bytes)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.data.to_bytes();
        bytes.extend_from_slice(&self.signature);
        bytes
    }
}
