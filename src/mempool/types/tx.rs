use crate::mempool::ports::{signing_key::SigningKey, verifying_key::VerifyingKey};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hash([u8; 32]);

pub struct Tx<VK>
where
    VK: VerifyingKey + Clone,
{
    prev_tx_hash: Hash,
    to: VK,
    from: VK,
    signature: VK::Signature,
}

impl<VK> Tx<VK>
where
    VK: VerifyingKey + Clone,
{
    pub fn new_signed<SK>(prev_tx_hash: Hash, from: VK, to: VK, sk: &SK) -> Self
    where
        SK: SigningKey<Signature = VK::Signature, VerifyingKey = VK>,
    {
        let msg = Self::msg(&prev_tx_hash, &to);
        let signature = sk.sign(&msg.0);

        Self {
            prev_tx_hash,
            to,
            from,
            signature,
        }
    }
    fn msg(prev_tx_hash: &Hash, to: &VK) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(prev_tx_hash.0);

        hasher.update(to.as_bytes());

        Hash(hasher.finalize().into())
    }
}
