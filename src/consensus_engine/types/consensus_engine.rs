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

    pub fn mine(&mut self, txs: Vec<Tx>, last_block: Block, hasher: &dyn Hasher) -> Block {
        let candidate =
            Block::new_generating_merkle_root(0, txs, last_block.header.prev_hash.clone(), hasher);
        self.miner.mine(candidate, self.difficulty)
    }
}
