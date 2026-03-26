use crate::block::Block;
use crate::consensus_engine::validation::block_validator::BlockValidator;

pub struct ChainValidator;

impl ChainValidator {
    pub fn validate(chain: &Vec<Block>) -> bool {
        if chain.is_empty() {
            return false;
        }

        if !BlockValidator::validate(&chain[0], None) {
            return false;
        }

        for i in 1..chain.len() {
            if !BlockValidator::validate(&chain[i], Some(&chain[i - 1])) {
                return false;
            }
        }

        true
    }
}
