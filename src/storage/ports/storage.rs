use crate::{common::types::tx::Hash, storage::types::storage_block::StorageBlock};

trait Storage {
    fn get_block(&self, hash: &Hash) -> Option<StorageBlock>;

    fn get_height(&self, hash: &Hash) -> Option<u64>;

    fn get_tip(&self) -> Option<StorageBlock>;

    fn insert_block(&mut self, block: StorageBlock) -> Result<(), String>;
}
