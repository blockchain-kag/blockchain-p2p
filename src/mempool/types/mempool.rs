use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
};

use crate::{
    common::{
        ports::hasher::Hasher,
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

    pub fn remove_included_txs(&mut self, block: &Block, hasher: Arc<dyn Hasher>) {
        let tx_hashes: HashSet<Hash> = block.txs.iter().map(|tx| tx.hash(hasher.clone())).collect();
        self.transactions
            .retain(|tx| !tx_hashes.contains(&tx.hash(hasher.clone())));
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
}
