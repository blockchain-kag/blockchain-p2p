use crate::consensus_engine::types::block::Block;

pub trait BlockValidator {
    fn validate(&self, prev_block: &Block, candidate_block: &Block) -> bool;
}
