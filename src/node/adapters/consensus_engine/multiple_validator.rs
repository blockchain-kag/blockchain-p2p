use crate::{
    common::types::block::Block, consensus_engine::ports::block_validator::BlockValidator,
};

#[derive(Default)]
pub struct MultipleValidator {
    validators: Vec<Box<dyn BlockValidator + Send + Sync>>,
}

impl BlockValidator for MultipleValidator {
    fn validate(&self, prev_block: &Block, candidate_block: &Block) -> bool {
        self.validators
            .iter()
            .all(|validator| validator.validate(prev_block, candidate_block))
    }
}

impl MultipleValidator {
    pub fn new() -> Self {
        Self::default()
    }
}
