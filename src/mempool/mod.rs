mod error;

pub use error::MempoolError;

use std::collections::HashMap;
use std::sync::Mutex;

use crate::transaction::Transaction;

pub const MAX_MEMPOOL_SIZE: usize = 100;

pub trait MempoolTrait {
    fn get_pending_transactions(&self) -> Vec<Transaction>;
    fn get_transactions(&self) -> Vec<Transaction>;
    fn remove_transactions(&self, tx: &Vec<Transaction>);
    fn add_transaction_to_mempool(&self, tx: &Transaction);
}

pub struct Mempool {
    transactions: Mutex<HashMap<String, Transaction>>,
    max_size: usize,
}

impl Mempool {
    pub fn new() -> Self {
        Self {
            transactions: Mutex::new(HashMap::new()),
            max_size: MAX_MEMPOOL_SIZE,
        }
    }

    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            transactions: Mutex::new(HashMap::new()),
            max_size,
        }
    }

    pub fn add_transaction(&self, tx: Transaction) -> Result<String, MempoolError> {
        Self::validate(&tx)?;

        let mut txs = self.transactions.lock().unwrap();

        if txs.len() >= self.max_size {
            return Err(MempoolError::Full);
        }

        let tx_id = tx.tx_id();
        if txs.contains_key(&tx_id) {
            return Err(MempoolError::Duplicate);
        }

        txs.insert(tx_id.clone(), tx);
        Ok(tx_id)
    }

    // No elimina las txs de la mempool — eso lo hace remove_transactions
    // una vez que el bloque fue minado y aceptado.
    pub fn get_transactions_for_block(&self, limit: usize) -> Vec<Transaction> {
        self.transactions.lock().unwrap().values().take(limit).cloned().collect()
    }

    pub fn get_size(&self) -> usize {
        self.transactions.lock().unwrap().len()
    }

    pub fn remove_transactions(&self, txs: &[Transaction]) {
        let mut pool = self.transactions.lock().unwrap();
        for tx in txs {
            pool.remove(&tx.tx_id());
        }
    }

    pub fn clear_invalid_transactions(&self) {
        self.transactions.lock().unwrap().retain(|_, tx| Self::validate(tx).is_ok());
    }

    fn validate(tx: &Transaction) -> Result<(), MempoolError> {
        if tx.from.is_empty() {
            return Err(MempoolError::InvalidTransaction("'from' vacío".into()));
        }
        if tx.to.is_empty() {
            return Err(MempoolError::InvalidTransaction("'to' vacío".into()));
        }
        if tx.from == tx.to {
            return Err(MempoolError::InvalidTransaction("'from' y 'to' iguales".into()));
        }
        if tx.amount == 0 {
            return Err(MempoolError::InvalidTransaction("monto cero".into()));
        }
        if tx.sig.is_empty() {
            return Err(MempoolError::InvalidTransaction("firma vacía".into()));
        }
        Ok(())
    }
}

impl MempoolTrait for Mempool {
    fn get_pending_transactions(&self) -> Vec<Transaction> {
        self.transactions.lock().unwrap().values().cloned().collect()
    }

    fn get_transactions(&self) -> Vec<Transaction> {
        self.transactions.lock().unwrap().values().cloned().collect()
    }

    fn remove_transactions(&self, txs: &Vec<Transaction>) {
        self.remove_transactions(txs.as_slice());
    }

    fn add_transaction_to_mempool(&self, tx: &Transaction) {
        let _ = self.add_transaction(tx.clone());
    }
}

impl Default for Mempool {
    fn default() -> Self {
        Self::new()
    }
}
