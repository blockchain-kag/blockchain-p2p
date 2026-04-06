pub trait Crypto: Send + Sync {
    fn verify(&self, pk: &[u8], msg: &[u8], sig: &[u8]) -> bool;
    fn sign(&self, sk: &[u8], msg: &[u8]) -> Vec<u8>;
}
