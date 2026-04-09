use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

use crate::common::ports::crypto::Crypto;

#[derive(Default)]
pub struct Ed25519Crypto {}

impl Ed25519Crypto {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Crypto for Ed25519Crypto {
    fn verify(&self, pk: &[u8], msg: &[u8], sig: &[u8]) -> bool {
        let pk: &[u8; 32] = match pk.try_into() {
            Ok(v) => v,
            Err(_) => return false,
        };

        let sig: &[u8; 64] = match sig.try_into() {
            Ok(v) => v,
            Err(_) => return false,
        };

        let verifying_key = match VerifyingKey::from_bytes(pk) {
            Ok(k) => k,
            Err(_) => return false,
        };

        let signature = Signature::from_bytes(sig);

        verifying_key.verify(msg, &signature).is_ok()
    }

    fn sign(&self, sk: &[u8], msg: &[u8]) -> Vec<u8> {
        let sk: &[u8; 32] = sk.try_into().unwrap();

        let signing_key = SigningKey::from_bytes(sk);

        let signature: Signature = signing_key.sign(msg);
        signature.to_bytes().to_vec()
    }
}
