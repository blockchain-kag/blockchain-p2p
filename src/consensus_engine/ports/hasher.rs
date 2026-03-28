use crate::common::types::tx::Hash;

pub trait Hasher {
    fn hash(&self, data: &[u8]) -> Hash;
}
