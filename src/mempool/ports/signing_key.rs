use crate::mempool::ports::verifying_key::VerifyingKey;

pub trait SigningKey {
    type Signature;
    type VerifyingKey: VerifyingKey<Signature = Self::Signature>;

    fn sign(&self, msg: &[u8]) -> Self::Signature;
    fn verifying_key(&self) -> Self::VerifyingKey;
}
