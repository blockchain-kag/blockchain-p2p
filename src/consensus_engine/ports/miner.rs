use crate::{common::ports::verifying_key::VerifyingKey, common::types::block::Block};

pub trait Miner<VK>
where
    VK: VerifyingKey + Clone,
{
    fn mine(&self, block: Block<VK>, difficulty: usize) -> Block<VK>;
}
