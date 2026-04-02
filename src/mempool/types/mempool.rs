use std::collections::VecDeque;

use crate::common::types::tx::Tx;

#[derive(Default)]
pub struct Mempool {
    transactions: VecDeque<Tx>,
}

impl Mempool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_first_n(&mut self, n: usize) -> VecDeque<Tx> {
        let mut result = VecDeque::new();

        for _ in 0..n {
            match self.transactions.pop_front() {
                Some(tx) => result.push_back(tx),
                None => break,
            }
        }

        result
    }

    pub fn push(&mut self, tx: Tx) {
        self.transactions.push_back(tx);
    }
}
