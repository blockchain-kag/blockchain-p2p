use std::{collections::HashMap, sync::Arc};

use crate::common::{
    ports::hasher::Hasher,
    types::{
        block::Block,
        tx::{Hash, Tx, TxOutput},
    },
};

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct UtxoKey(pub Hash, pub usize);

pub trait Storage: Send {
    // blocks
    fn get_block(&self, hash: &Hash) -> Option<&Block>;
    fn get_height(&self, hash: &Hash) -> Option<u64>;
    fn get_tip(&self) -> Option<&Block>;
    fn insert_block(&mut self, block: Block, hasher: Arc<dyn Hasher>) -> Result<(), String>;

    // UTXO
    fn get_utxo_map(&self) -> &HashMap<UtxoKey, TxOutput>;
    fn get_utxo(&self, key: &UtxoKey) -> Option<&TxOutput>;
    fn contains_utxo(&self, key: &UtxoKey) -> bool;
    fn apply_tx(&mut self, tx: &Tx, hasher: Arc<dyn Hasher>);
}
