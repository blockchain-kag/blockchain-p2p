use crate::common::ports::signature::Signature;

pub trait VerifyingKey {
    type Signature: Signature;
    fn verify(&self, message: &[u8], signature: &Self::Signature) -> bool;
    fn as_bytes(&self) -> &[u8];
}
