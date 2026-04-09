use std::{
    fmt::{Display, Formatter, Result},
    sync::Arc,
};

use crate::common::ports::{crypto::Crypto, hasher::Hasher};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Hash(pub [u8; 32]);

impl Display for Hash {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl Hash {
    pub fn zero() -> Hash {
        Hash([0; 32])
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TxOutput {
    pub amount: u64,
    pub recipient: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TxInput {
    pub prev_tx: Hash,
    pub output_index: usize,
    pub signature: Vec<u8>,
    pub pubkey: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Tx {
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
}

impl Tx {
    pub fn hash(&self, hasher: Arc<dyn Hasher>) -> Hash {
        hasher.hash(&self.to_bytes())
    }
    pub fn new(inputs: Vec<TxInput>, outputs: Vec<TxOutput>) -> Self {
        Self { inputs, outputs }
    }

    pub fn sign(self, hasher: Arc<dyn Hasher>, crypto: Arc<dyn Crypto>) -> Self {
        let msg = self.hash(hasher).0;
        let inputs = self
            .inputs
            .iter()
            .map(|input| TxInput {
                prev_tx: input.prev_tx,
                output_index: input.output_index,
                signature: crypto.sign(input.pubkey.as_ref(), msg.as_ref()),
                pubkey: input.pubkey.clone(),
            })
            .collect();
        Tx {
            inputs,
            outputs: self.outputs,
        }
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
