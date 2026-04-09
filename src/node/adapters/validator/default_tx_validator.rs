use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{
    common::{
        ports::{crypto::Crypto, hasher::Hasher},
        types::tx::{Tx, TxOutput},
    },
    node::ports::storage::UtxoKey,
    validator::ports::tx_validator::TxValidator,
};

pub struct DefaultTxValidator {
    crypto: Arc<dyn Crypto>,
    hasher: Arc<dyn Hasher>,
}

impl DefaultTxValidator {
    pub fn new(crypto: Arc<dyn Crypto>, hasher: Arc<dyn Hasher>) -> Self {
        Self { crypto, hasher }
    }
}

impl TxValidator for DefaultTxValidator {
    fn validate(
        &self,
        tx: &Tx,
        utxo_set: &HashMap<UtxoKey, TxOutput>,
        spent_set: &HashSet<UtxoKey>,
    ) -> bool {
        any_input_and_output(tx)
            && does_input_exists_and_is_unique(tx, utxo_set, spent_set)
            && inputs_cover_outputs(tx, utxo_set)
            && check_transactions_signature(tx, utxo_set, self.crypto.clone(), self.hasher.clone())
    }
}

fn check_transactions_signature(
    tx: &Tx,
    utxo_map: &HashMap<UtxoKey, TxOutput>,
    crypto: Arc<dyn Crypto>,
    hasher: Arc<dyn Hasher>,
) -> bool {
    tx.inputs.iter().all(|input| {
        let key = UtxoKey(input.prev_tx, input.output_index);

        let utxo = match utxo_map.get(&key) {
            Some(u) => u,
            None => return false,
        };

        if input.pubkey != utxo.recipient {
            return false;
        }

        let message = tx.hash(hasher.clone());

        crypto.verify(&input.pubkey, &message.0, &input.signature)
    })
}

fn any_input_and_output(tx: &Tx) -> bool {
    !tx.inputs.is_empty() && !tx.outputs.is_empty()
}

fn does_input_exists_and_is_unique(
    tx: &Tx,
    utxo_map: &HashMap<UtxoKey, TxOutput>,
    mempool_spent: &HashSet<UtxoKey>,
) -> bool {
    let mut seen = HashSet::new();

    for input in &tx.inputs {
        let key = UtxoKey(input.prev_tx, input.output_index);

        if !seen.insert(key) {
            return false;
        }

        if !utxo_map.contains_key(&key) {
            return false;
        }

        if mempool_spent.contains(&key) {
            return false;
        }
    }

    true
}

fn inputs_cover_outputs(tx: &Tx, utxo_map: &HashMap<UtxoKey, TxOutput>) -> bool {
    let mut input_sum = 0u64;

    for input in &tx.inputs {
        let key = UtxoKey(input.prev_tx, input.output_index);

        let utxo = match utxo_map.get(&key) {
            Some(v) => v,
            None => return false,
        };

        input_sum = match input_sum.checked_add(utxo.amount) {
            Some(v) => v,
            None => return false,
        };
    }

    let mut output_sum = 0u64;

    for output in &tx.outputs {
        if output.amount == 0 {
            return false;
        }

        output_sum = match output_sum.checked_add(output.amount) {
            Some(v) => v,
            None => return false,
        };
    }

    input_sum >= output_sum
}
