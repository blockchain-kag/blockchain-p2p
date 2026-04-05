use crate::common::{
    ports::hasher::Hasher,
    types::{block::Block, tx::Hash},
};

pub trait Storage {
    fn get_block(&self, hash: &Hash) -> Option<&Block>;

    fn get_height(&self, hash: &Hash) -> Option<u64>;

    fn get_tip(&self) -> Option<&Block>;

    fn insert_block(&mut self, block: Block, hasher: &dyn Hasher) -> Result<(), String>;
}
