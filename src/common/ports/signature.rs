use serde::{Deserialize, Serialize};

pub trait Signature: Serialize + for<'de> Deserialize<'de> {
    fn to_bytes(&self) -> Vec<u8>;
}
