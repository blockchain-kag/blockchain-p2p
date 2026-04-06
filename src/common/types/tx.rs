use crate::common::ports::hasher::Hasher;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    pub fn zero() -> Hash {
        Hash([0; 32])
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TxOutput {
    pub amount: u64,
    pub recipient: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TxInput {
    pub prev_tx: Hash,
    pub output_index: usize,
    pub signature: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Tx {
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
}

impl Tx {
    pub fn hash(&self, hasher: &dyn Hasher) -> Hash {
        hasher.hash(&self.to_bytes())
    }
    pub fn new(inputs: Vec<TxInput>, outputs: Vec<TxOutput>) -> Self {
        Self { inputs, outputs }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        for input in &self.inputs {
            bytes.extend_from_slice(&input.prev_tx.0);
            bytes.extend_from_slice(&input.output_index.to_be_bytes());
        }

        for output in &self.outputs {
            bytes.extend_from_slice(&output.recipient);
            bytes.extend_from_slice(&output.amount.to_be_bytes());
        }

        bytes
    }
}
