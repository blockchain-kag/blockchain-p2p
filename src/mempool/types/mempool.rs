use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    common::{
        ports::{crypto::Crypto, hasher::Hasher},
        types::{
            block::Block,
            tx::{Hash, Tx, TxOutput},
        },
    },
    node::ports::storage::UtxoKey,
    validator::ports::tx_validator::TxValidator,
};

pub struct Mempool {
    transactions: VecDeque<Tx>,
    spent: HashSet<UtxoKey>,
    tx_validator: Box<dyn TxValidator>,
}

impl Mempool {
    pub fn new(tx_validator: Box<dyn TxValidator>) -> Self {
        Self {
            transactions: VecDeque::new(),
            spent: HashSet::new(),
            tx_validator,
        }
    }

    pub fn peek_first_n(&self, n: usize) -> VecDeque<Tx> {
        self.transactions.iter().take(n).cloned().collect()
    }

    pub fn remove_included_txs(&mut self, block: &Block, hasher: &dyn Hasher) {
        let tx_hashes: HashSet<Hash> = block.txs.iter().map(|tx| tx.hash(hasher)).collect();
        self.transactions
            .retain(|tx| !tx_hashes.contains(&tx.hash(hasher)));
        self.spent.clear();
        for tx in &self.transactions {
            for input in &tx.inputs {
                self.spent
                    .insert(UtxoKey(input.prev_tx, input.output_index));
            }
        }
    }

    pub fn push(&mut self, tx: Tx, utxo_set: &HashMap<UtxoKey, TxOutput>) {
        if !self.tx_validator.validate(&tx, utxo_set, &self.spent) {
            return;
        }

        for input in &tx.inputs {
            self.spent
                .insert(UtxoKey(input.prev_tx, input.output_index));
        }

        self.transactions.push_back(tx);
    }

    pub fn push_n(&mut self, txs: Vec<Tx>, utxo_map: &HashMap<UtxoKey, TxOutput>) {
        for tx in txs {
            self.push(tx, utxo_map)
        }
    }

    pub fn is_tx_valid(
        &self,
        tx: &Tx,
        utxo_map: &HashMap<UtxoKey, TxOutput>,
        crypto: &dyn Crypto,
        hasher: &dyn Hasher,
    ) -> bool {
        any_input_and_output(tx)
            && is_transfering_something(tx)
            && does_input_exists_and_is_unique(tx, utxo_map, &self.spent)
            && has_superavit(tx, utxo_map)
            && check_transactions_signature(tx, utxo_map, crypto, hasher)
    }
}

fn check_transactions_signature(
    tx: &Tx,
    utxo_map: &HashMap<UtxoKey, TxOutput>,
    crypto: &dyn Crypto,
    hasher: &dyn Hasher,
) -> bool {
    let message = tx.hash(hasher);

    tx.inputs.iter().all(|input| {
        let key = UtxoKey(input.prev_tx, input.output_index);

        let utxo = match utxo_map.get(&key) {
            Some(u) => u,
            None => return false,
        };

        if input.pubkey != utxo.recipient {
            return false;
        }

        crypto.verify(&input.pubkey, &message.0, &input.signature)
    })
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
