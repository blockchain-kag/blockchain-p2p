use crate::common::types::crypto_scheme::CryptoScheme;

pub trait SigningKey {
    fn sign(&self, msg: &[u8]) -> Vec<u8>;
    fn verifying_key(&self) -> Vec<u8>;
    fn scheme(&self) -> CryptoScheme;
}
