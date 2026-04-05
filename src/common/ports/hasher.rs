use crate::common::types::tx::Hash;

pub trait Hasher: Send + Sync {
    fn hash(&self, data: &[u8]) -> Hash;
}
