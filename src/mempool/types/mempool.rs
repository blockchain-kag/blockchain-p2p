use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    common::types::tx::{Tx, TxOutput},
    node::ports::storage::UtxoKey,
};

#[derive(Default)]
pub struct Mempool {
    transactions: VecDeque<Tx>,
    spent: HashSet<UtxoKey>,
}

impl Mempool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_first_n(&mut self, n: usize) -> VecDeque<Tx> {
        let mut result = VecDeque::new();

        for _ in 0..n {
            match self.transactions.pop_front() {
                Some(tx) => {
                    for input in &tx.inputs {
                        self.spent
                            .remove(&UtxoKey(input.prev_tx, input.output_index));
                    }

                    result.push_back(tx);
                }
                None => break,
            }
        }

        result
    }

    pub fn push(&mut self, tx: Tx, utxo_map: &HashMap<UtxoKey, TxOutput>) -> bool {
        if !self.is_tx_valid(&tx, utxo_map) {
            return false;
        }

        for input in &tx.inputs {
            self.spent
                .insert(UtxoKey(input.prev_tx, input.output_index));
        }

        self.transactions.push_back(tx);
        true
    }

    pub fn is_tx_valid(&self, tx: &Tx, utxo_map: &HashMap<UtxoKey, TxOutput>) -> bool {
        any_input_and_output(tx)
            && is_transfering_something(tx)
            && does_input_exists_and_is_unique(tx, utxo_map, &self.spent)
            && has_superavit(tx, utxo_map)
    }
}
fn any_input_and_output(tx: &Tx) -> bool {
    !tx.inputs.is_empty() && !tx.outputs.is_empty()
}

fn is_transfering_something(tx: &Tx) -> bool {
    tx.outputs.iter().all(|output| output.amount > 0)
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

fn has_superavit(tx: &Tx, utxo_map: &HashMap<UtxoKey, TxOutput>) -> bool {
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
