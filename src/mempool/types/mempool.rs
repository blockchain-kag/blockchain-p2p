use std::collections::VecDeque;

use crate::storage::types::storage_tx::StorageTx;

pub struct Mempool {
    transactions: VecDeque<StorageTx>,
}

impl Mempool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_first_n(&mut self, n: usize) -> VecDeque<StorageTx> {
        let mut result = VecDeque::new();

        for _ in 0..n {
            match self.transactions.pop_front() {
                Some(tx) => result.push_back(tx),
                None => break,
            }
        }

        result
    }

    pub fn push(&mut self, tx: StorageTx) {
        self.transactions.push_back(tx);
    }
}

impl Default for Mempool {
    fn default() -> Self {
        Self {
            transactions: VecDeque::default(),
        }
    }
}
