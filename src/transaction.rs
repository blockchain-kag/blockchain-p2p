use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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

    pub fn tx_id(&self) -> String {
        let data = format!("{}|{}|{}|{}", self.from, self.to, self.amount, self.sig);
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
