use std::collections::VecDeque;

use crate::mempool::{ports::verifying_key::VerifyingKey, types::tx::Tx};

pub struct Mempool<VK: Clone>
where
    VK: VerifyingKey,
{
    transactions: VecDeque<Tx<VK>>,
}

impl<VK: Clone> Mempool<VK>
where
    VK: VerifyingKey,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_first_n(&mut self, n: usize) -> VecDeque<Tx<VK>> {
        let mut result = VecDeque::new();

        for _ in 0..n {
            match self.transactions.pop_front() {
                Some(tx) => result.push_back(tx),
                None => break,
            }
        }

        result
    }

    pub fn push(&mut self, tx: Tx<VK>) {
        self.transactions.push_back(tx);
    }
}

impl<VK: Clone> Default for Mempool<VK>
where
    VK: VerifyingKey,
{
    fn default() -> Self {
        Self {
            transactions: VecDeque::new(),
        }
    }
}
