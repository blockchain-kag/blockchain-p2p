use crate::common::{ports::verifying_key::VerifyingKey, types::block::Block};

pub trait BlockValidator<VK>
where
    VK: VerifyingKey + Clone,
{
    fn validate(&self, prev_block: &Block<VK>, candidate_block: &Block<VK>) -> bool;
}
