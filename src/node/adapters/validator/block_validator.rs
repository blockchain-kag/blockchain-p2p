use std::collections::{HashMap, HashSet};

use crate::{
    common::{
        ports::hasher::Hasher,
        types::{
            block::Block,
            tx::{Hash, Tx, TxOutput},
        },
    },
    node::ports::storage::UtxoKey,
    validator::ports::{block_validator::BlockValidator, tx_validator::TxValidator},
};

pub struct DefaultBlockValidator<'a> {
    tx_validator: Box<dyn TxValidator>,
    hasher: &'a dyn Hasher,
}

impl<'a> DefaultBlockValidator<'a> {
    pub fn new(tx_validator: Box<dyn TxValidator>, hasher: &'a dyn Hasher) -> Self {
        Self {
            tx_validator,
            hasher,
        }
    }
}

const DIFFICULTY: usize = 3;
const BLOCK_SUBSIDY: u64 = 50;

impl<'a> BlockValidator for DefaultBlockValidator<'a> {
    fn validate(
        &self,
        block: &Block,
        prev_block: &Block,
        utxo_map: &HashMap<UtxoKey, TxOutput>,
    ) -> bool {
        // prev hash
        if block.header.prev_hash != prev_block.hash(self.hasher) {
            return false;
        }

        // at least one tx
        if block.txs.is_empty() {
            return false;
        }

        // pow
        let hash = block.hash(self.hasher);

        if !hash.0.starts_with(&[0].repeat(DIFFICULTY)) {
            return false;
        }

        // timestamp
        if block.header.timestamp <= prev_block.header.timestamp {
            return false;
        }

        // merkle root
        if merkle_root(&block.txs, self.hasher) != block.header.merkle_root {
            return false;
        }

        if block
            .txs
            .iter()
            .filter(|tx| tx.inputs.is_empty()) // or your coinbase condition
            .count()
            != 1
        {
            return false;
        };

        let coinbase = &block.txs[0];

        if !coinbase.inputs.is_empty() {
            return false;
        }
        if coinbase.outputs.is_empty() {
            return false;
        }

        let mut working_utxo = utxo_map.clone();
        let mut spent_in_block = HashSet::new();

        let mut total_fees = 0u64;
        for (i, tx) in block.txs.iter().enumerate() {
            if i == 0 {
                continue;
            }
            // validate tx
            if !self
                .tx_validator
                .validate(tx, &working_utxo, &spent_in_block)
            {
                return false;
            }

            // spend inputs
            let mut input_sum = 0u64;
            for input in &tx.inputs {
                let key = UtxoKey(input.prev_tx, input.output_index);

                if spent_in_block.contains(&key) {
                    return false;
                }

                spent_in_block.insert(key);
                let utxo = match working_utxo.get(&key) {
                    Some(u) => u,
                    None => return false,
                };

                input_sum = match input_sum.checked_add(utxo.amount) {
                    Some(v) => v,
                    None => return false,
                };
                working_utxo.remove(&key);
            }

            // add outputs
            let mut output_sum = 0u64;
            for (i, output) in tx.outputs.iter().enumerate() {
                let key = UtxoKey(tx.hash(self.hasher), i);
                output_sum = match output_sum.checked_add(output.amount) {
                    Some(v) => v,
                    None => return false,
                };
                working_utxo.insert(key, output.clone());
            }
            let fee = match input_sum.checked_sub(output_sum) {
                Some(f) => f,
                None => return false,
            };
            total_fees = match total_fees.checked_add(fee) {
                Some(v) => v,
                None => return false,
            };
        }

        let mut coinbase_output_sum = 0u64;
        for o in &coinbase.outputs {
            coinbase_output_sum = match coinbase_output_sum.checked_add(o.amount) {
                Some(v) => v,
                None => return false,
            };
        }
        if coinbase_output_sum > BLOCK_SUBSIDY + total_fees {
            return false;
        }
        true
    }
}

fn merkle_root(txs: &[Tx], hasher: &dyn Hasher) -> Hash {
    let mut hashes: Vec<Hash> = txs.iter().map(|tx| tx.hash(hasher)).collect();

    while hashes.len() > 1 {
        let mut next = vec![];

        for i in (0..hashes.len()).step_by(2) {
            let a = hashes[i];
            let b = if i + 1 < hashes.len() {
                hashes[i + 1]
            } else {
                hashes[i] // duplicate if odd
            };
            next.push(hash_pair(a, b, hasher));
        }

        hashes = next;
    }

    hashes[0]
}

fn hash_pair(a: Hash, b: Hash, hasher: &dyn Hasher) -> Hash {
    let mut data = Vec::with_capacity(a.0.len() + b.0.len());

    data.extend_from_slice(&a.0);
    data.extend_from_slice(&b.0);

    hasher.hash(&data)
}
