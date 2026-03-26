use crate::block::Block;

pub trait Hasher {
    fn hash_block(block: &Block) -> String;
}

pub struct Sha256Hasher;

impl Hasher for Sha256Hasher {
    fn hash_block(block: &Block) -> String {
        block.calculate_hash()
    }
}
