pub trait VerifyingKey {
    type Signature;
    fn verify(&self, message: &[u8], signature: &Self::Signature) -> bool;
    fn as_bytes(&self) -> &[u8];
}
