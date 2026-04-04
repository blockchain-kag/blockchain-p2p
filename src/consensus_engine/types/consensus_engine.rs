use std::collections::VecDeque;

use crate::common::ports::hasher::Hasher;
use crate::common::types::block::Block;
use crate::common::types::tx::Tx;
use crate::consensus_engine::ports::block_validator::BlockValidator;
use crate::consensus_engine::ports::miner::Miner;

pub struct ConsensusEngine {
    miner: Box<dyn Miner>,
    validator: Box<dyn BlockValidator>,
    difficulty: usize,
}

impl ConsensusEngine {
    pub fn new(
        miner: Box<dyn Miner>,
        validator: Box<dyn BlockValidator>,
        difficulty: usize,
    ) -> Self {
        Self {
            miner,
            validator,
            difficulty,
        }
    }

    pub fn validate(&self, prev_block: &Block, candidate_block: &Block) -> bool {
        self.validator.validate(prev_block, candidate_block)
    }

    pub fn start_mining(
        &self,
        txs: VecDeque<Tx>,
        last_block: &Block,
        hasher: &dyn Hasher,
    ) -> Result<(), ()> {
        todo!();
    }

    pub fn stop_mining(&self) -> Result<(), ()> {
        todo!();
    }
    pub fn pause_mining(&self) -> Result<(), ()> {
        todo!();
    }
    pub fn continue_mining(&self) -> Result<(), ()> {
        todo!();
    }
}
