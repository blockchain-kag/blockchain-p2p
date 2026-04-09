use std::collections::{HashMap, HashSet};

use crate::{
    common::types::tx::{Tx, TxOutput},
    node::ports::storage::UtxoKey,
};

pub trait TxValidator: Send + Sync {
    fn validate(
        &self,
        tx: &Tx,
        utxo_set: &HashMap<UtxoKey, TxOutput>,
        spent_set: &HashSet<UtxoKey>,
    ) -> bool;
}
