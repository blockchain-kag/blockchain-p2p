pub trait Wallet: Send + Sync {
    fn sign(&self, pubkey: &[u8], message: &[u8]) -> Vec<u8>;
    fn verify(&self, pubkey: &[u8], message: &[u8], signature: &[u8]) -> bool;
    fn change_address(&self) -> Vec<u8>;
}
