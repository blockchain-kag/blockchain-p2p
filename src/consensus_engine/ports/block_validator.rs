use crate::common::types::block::Block;

pub trait BlockValidator: Send + Sync {
    fn validate(&self, prev_block: &Block, candidate_block: &Block) -> bool;
}
