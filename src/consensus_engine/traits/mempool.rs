use crate::transaction::Transaction;

pub trait Mempool: Send {
    fn get_pending_transactions(&self) -> Vec<Transaction>;
    fn get_transactions(&self) -> Vec<Transaction>;
    fn remove_transactions(&self, tx: &Vec<Transaction>);
    fn add_transaction_to_mempool(&self, tx: &Transaction);
}
