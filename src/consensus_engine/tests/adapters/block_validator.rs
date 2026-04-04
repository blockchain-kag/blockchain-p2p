use crate::consensus_engine::ports::block_validator::BlockValidator;

pub struct ValidBlockValidator();

impl BlockValidator for ValidBlockValidator {
    fn validate(
        &self,
        _prev_block: &crate::common::types::block::Block,
        _candidate_block: &crate::common::types::block::Block,
    ) -> bool {
        true
    }
}
