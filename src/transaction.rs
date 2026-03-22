use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Representa una transacción pendiente en la red.
/// La firma (sig) es provista por el cliente que origina la transacción.
/// El nodo NO verifica criptográficamente la firma en este TP — solo
/// valida que no esté vacía (presencia estructural).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub sig: String,
}

impl Transaction {
    pub fn new(from: String, to: String, amount: u64, sig: String) -> Self {
        Transaction {
            from,
            to,
            amount,
            sig,
        }
    }

    /// Genera un ID único para esta transacción basado en su contenido.
    /// Dos transacciones con los mismos campos producen el mismo tx_id.
    /// La mempool usa esto para detectar duplicados.
    pub fn tx_id(&self) -> String {
        let data = format!("{}{}{}{}", self.from, self.to, self.amount, self.sig);
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
