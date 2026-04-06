use std::sync::Arc;

use crate::common::ports::crypto::Crypto;

pub struct Wallet {
    crypto: Arc<dyn Crypto>,
    key: KeyPair,
}

pub struct KeyPair {
    pub pk: Vec<u8>,
    pub sk: Vec<u8>,
}

impl Wallet {
    pub fn new(crypto: Arc<dyn Crypto>) -> Self {
        Self {
            crypto,
            key: KeyPair {
                pk: vec![],
                sk: vec![],
            },
        }
    }

    pub fn change_address(&self) -> Vec<u8> {
        vec![]
    }

    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.crypto.sign(&self.key.sk, message)
    }

    pub fn validate(&self, msg: &[u8], sig: &[u8]) -> bool {
        self.crypto.verify(&self.key.pk, msg, sig)
    }
}
