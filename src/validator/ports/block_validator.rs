use std::collections::HashMap;

use crate::{
    common::types::{block::Block, tx::TxOutput},
    node::ports::storage::UtxoKey,
};

pub trait BlockValidator: Send + Sync {
    fn validate(
        &self,
        block: &Block,
        prev_block: &Block,
        utxo_map: &HashMap<UtxoKey, TxOutput>,
    ) -> bool;
}
