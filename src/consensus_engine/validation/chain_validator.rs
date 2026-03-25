use crate::consensus_engine::block::block::Block;
use crate::consensus_engine::validation::block_validator::BlockValidator;

pub struct ChainValidator;

impl ChainValidator {
    pub fn validate(chain: &Vec<Block>) -> bool {
        for i in 1..chain.len() {
            let current = &chain[i];
            let previous = &chain[i - 1];

            return BlockValidator::validate(current, previous);
        }
        true
    }
}