use serde::{
   Deserialize, 
   Serialize
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
   pub(crate) from: String,
   pub(crate) to: String,
   pub(crate) amount: u64,
   pub(crate) sig: String,
}