use crate::common::ports::verifying_key::VerifyingKey;
use crate::common::types::block::Block;
use crate::common::types::tx::Tx;
use crate::consensus_engine::ports::block_validator::BlockValidator;
use crate::consensus_engine::ports::hasher::Hasher;
use crate::consensus_engine::ports::miner::Miner;

pub struct ConsensusEngine<VK>
where
    VK: VerifyingKey + Clone,
{
    miner: Box<dyn Miner<VK>>,
    validator: Box<dyn BlockValidator<VK>>,
    difficulty: usize,
}

impl<VK> ConsensusEngine<VK>
where
    VK: VerifyingKey + Clone,
{
    pub fn new(
        miner: Box<dyn Miner<VK>>,
        validator: Box<dyn BlockValidator<VK>>,
        difficulty: usize,
    ) -> Self {
        Self {
            miner,
            validator,
            difficulty,
        }
    }

    pub fn validate(&self, prev_block: &Block<VK>, candidate_block: &Block<VK>) -> bool {
        self.validator.validate(prev_block, candidate_block)
    }

    pub fn mine(
        &mut self,
        txs: Vec<Tx<VK>>,
        last_block: Block<VK>,
        hasher: &dyn Hasher,
    ) -> Block<VK> {
        let candidate =
            Block::new_generating_merkle_root(0, txs, last_block.header.prev_hash.clone(), hasher);
        self.miner.mine(candidate, self.difficulty)
    }
}
