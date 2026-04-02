use crate::common::types::tx::Hash;

pub struct StorageTx {
    pub prev_tx_hash: Hash,
    pub sender: Vec<u8>,
    pub recipient: Vec<u8>,
    pub amount: u64,
    pub signature: Vec<u8>,
}

impl StorageTx {
    fn new(
        prev_tx_hash: Hash,
        sender: Vec<u8>,
        recipient: Vec<u8>,
        amount: u64,
        signature: Vec<u8>,
    ) -> Self {
        Self {
            prev_tx_hash,
            sender,
            recipient,
            amount,
            signature,
        }
    }
}
