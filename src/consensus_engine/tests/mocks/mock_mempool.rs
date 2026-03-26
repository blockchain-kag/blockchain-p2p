use crate::transaction::Transaction;
use crate::consensus_engine::traits::mempool::Mempool;

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
        self.txs.clone()
    }

    fn get_transactions(&self) -> Vec<Transaction> {
        self.txs.clone()
    }

    fn remove_transactions(&self, _tx: &Vec<Transaction>) {}

    fn add_transaction_to_mempool(&self, _tx: &Transaction) {}
}
