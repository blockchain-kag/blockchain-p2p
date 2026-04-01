use crate::{
    common::ports::hasher::Hasher,
    common::ports::{signature::Signature, signing_key::SigningKey, verifying_key::VerifyingKey},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Hash(pub [u8; 32]);

#[derive(Serialize, Deserialize)]
struct TxData<VK> {
    prev_tx_hash: Hash,
    from: VK,
    to: VK,
    amount: u64,
}

impl<VK> TxData<VK>
where
    VK: VerifyingKey,
{
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.prev_tx_hash.0);
        bytes.extend_from_slice(self.from.as_bytes());
        bytes.extend_from_slice(self.to.as_bytes());
        bytes.extend_from_slice(&self.amount.to_be_bytes());

        bytes
    }
}

#[derive(Serialize, Deserialize)]
pub struct Tx<VK>
where
    VK: VerifyingKey + Clone,
{
    data: TxData<VK>,
    signature: VK::Signature,
}

impl<VK> Tx<VK>
where
    VK: VerifyingKey + Clone,
{
    pub fn new_signed<SK>(
        prev_tx_hash: Hash,
        from: VK,
        to: VK,
        amount: u64,
        sk: &SK,
        hasher: &dyn Hasher,
    ) -> Self
    where
        SK: SigningKey<Signature = VK::Signature, VerifyingKey = VK>,
    {
        let data = TxData {
            prev_tx_hash,
            to,
            from,
            amount,
        };
        let msg = Self::msg(hasher, &data);
        let signature = sk.sign(&msg.0);

        Self { data, signature }
    }

    pub fn verify(&self, hasher: &dyn Hasher) -> bool {
        let msg = Self::msg(hasher, &self.data);
        self.data.from.verify(&msg.0, &self.signature)
    }

    fn msg(hasher: &dyn Hasher, data: &TxData<VK>) -> Hash {
        let bytes = data.to_bytes();
        hasher.hash(&bytes)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.data.to_bytes();
        bytes.extend_from_slice(self.signature.to_bytes().as_slice());
        bytes
    }
}
