use serde::de::Unexpected::Option;
use crate::consensus_engine::block::block::Block;
use crate::consensus_engine::validation::block_validator::BlockValidator;

pub struct ChainValidator;


impl ChainValidator {
    pub fn validate(chain: &Vec<Block>) -> bool {
        if chain.is_empty() { return false; }

        if !BlockValidator::validate(&chain[0], None) { return false; }

        for i in 1..chain.len() {
            let current = &chain[i];
            let previous = &chain[i - 1];

            if !BlockValidator::validate(current, Some(previous)) { return false; }
        }

        true
    }
}