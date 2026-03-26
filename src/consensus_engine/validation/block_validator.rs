use crate::block::Block;

pub struct BlockValidator;

impl BlockValidator {
    pub fn validate(current: &Block, previous: Option<&Block>) -> bool {
        match previous {
            None => {
                current.index == 0
                    && current.previous_hash.is_empty()
                    && current.hash == current.calculate_hash()
            }
            Some(previous) => {
                current.hash == current.calculate_hash()
                    && current.previous_hash == previous.hash
                    && current.index == previous.index + 1
            }
        }
    }
}
