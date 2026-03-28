use crate::consensus_engine::ports::block_validator::BlockValidator;
use crate::consensus_engine::ports::hasher::Hasher;
use crate::consensus_engine::ports::miner::Miner;
use crate::consensus_engine::types::block::Block;
use crate::consensus_engine::types::tx::Tx;

pub struct Engine {
    miner: Box<dyn Miner>,
    validator: Box<dyn BlockValidator>,
    difficulty: usize,
}

impl Engine {
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

    pub fn mine(
        &mut self,
        txs: Vec<Tx>,
        last_block: Block,
        hasher: Box<dyn Hasher>,
    ) -> Option<Block> {
        let candidate = Block::new_generating_merkle_root(
            0,
            txs.clone(),
            last_block.header.prev_hash.clone(),
            hasher,
        );
        let mined = self.miner.mine(candidate, self.difficulty);
        if self.validate(&last_block, &mined) {
            Some(mined)
        } else {
            None
        }
    }
}
