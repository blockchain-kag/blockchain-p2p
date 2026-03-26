use crate::consensus_engine::traits::mempool::Mempool;
use crate::consensus_engine::transaction::transaction::Transaction;

pub struct MockMempool {
    pub txs: Vec<Transaction>,
}

impl MockMempool {
    pub fn new(txs: Vec<Transaction>) -> Self {
        Self { txs }
    }
}

impl Mempool for MockMempool {
    fn get_pending_transactions(&self) -> Vec<Transaction> {
        todo!()
    }

    fn get_transactions(&self) -> Vec<Transaction> {
        todo!()
    }

    fn remove_transactions(&self, tx: &Vec<Transaction>) {
        todo!()
    }

    fn add_transaction_to_mempool(&self, tx: &Transaction) {
        todo!()
    }
}