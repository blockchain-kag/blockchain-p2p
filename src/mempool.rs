use std::collections::HashMap;

use crate::transaction::Transaction;

pub const MAX_MEMPOOL_SIZE: usize = 100;

#[derive(Debug, PartialEq)]
pub enum MempoolError {
    Full,
    Duplicate,
    InvalidTransaction(String),
}

impl std::fmt::Display for MempoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MempoolError::Full => write!(f, "Mempool llena"),
            MempoolError::Duplicate => write!(f, "Transacción duplicada"),
            MempoolError::InvalidTransaction(reason) => {
                write!(f, "Transacción inválida: {}", reason)
            }
        }
    }
}

pub struct Mempool {
    transactions: HashMap<String, Transaction>,
    max_size: usize,
}

impl Mempool {
    pub fn new() -> Self {
        Self {
            transactions: HashMap::new(),
            max_size: MAX_MEMPOOL_SIZE,
        }
    }

    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            transactions: HashMap::new(),
            max_size,
        }
    }

    pub fn add_transaction(&mut self, tx: Transaction) -> Result<String, MempoolError> {
        Self::validate(&tx)?;

        if self.transactions.len() >= self.max_size {
            return Err(MempoolError::Full);
        }

        let tx_id = tx.tx_id();
        if self.transactions.contains_key(&tx_id) {
            return Err(MempoolError::Duplicate);
        }

        self.transactions.insert(tx_id.clone(), tx);
        Ok(tx_id)
    }

    // No elimina las txs de la mempool — eso lo hace remove_transactions
    // una vez que el bloque fue minado y aceptado.
    pub fn get_transactions_for_block(&self, limit: usize) -> Vec<Transaction> {
        self.transactions.values().take(limit).cloned().collect()
    }

    pub fn remove_transactions(&mut self, transactions: &[Transaction]) {
        for tx in transactions {
            self.transactions.remove(&tx.tx_id());
        }
    }

    pub fn get_size(&self) -> usize {
        self.transactions.len()
    }

    pub fn clear_invalid_transactions(&mut self) {
        self.transactions.retain(|_, tx| Self::validate(tx).is_ok());
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

impl Default for Mempool {
    fn default() -> Self {
        Self::new()
    }
}
