use sha2::{Digest, Sha256};

use crate::common::{ports::hasher::Hasher, types::tx::Hash};

pub struct Sha256Hasher;

impl Hasher for Sha256Hasher {
    fn hash(&self, data: &[u8]) -> crate::common::types::tx::Hash {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();

        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);

        Hash(bytes)
    }
}
