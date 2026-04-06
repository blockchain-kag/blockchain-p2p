use std::collections::{HashMap, VecDeque};

use crate::{
    common::{
        ports::hasher::Hasher,
        types::{
            block::Block,
            tx::{Hash, Tx, TxOutput},
        },
    },
    node::ports::storage::{Storage, UtxoKey},
};

#[derive(Default)]
pub struct InMemoryStorage {
    height: HashMap<Hash, u64>,
    blocks: HashMap<Hash, Block>,
    tip: Option<Hash>,
    utxo_map: HashMap<UtxoKey, TxOutput>,
}

impl InMemoryStorage {
    pub fn new(hasher: &dyn Hasher) -> Self {
        let genesis_block = Block::new(0, Hash::zero(), 0, VecDeque::new(), hasher);
        let hash = genesis_block.hash(hasher);
        Self {
            height: HashMap::from([(hash, 0)]),
            blocks: HashMap::from([(hash, genesis_block.clone())]),
            tip: Some(hash),
            utxo_map: from_queue_to_utxo_map(genesis_block.txs, hasher),
        }
    }
}

fn from_queue_to_utxo_map(queue: VecDeque<Tx>, hasher: &dyn Hasher) -> HashMap<UtxoKey, TxOutput> {
    let mut map = HashMap::new();
    for tx in queue {
        let hash = tx.hash(hasher);
        for (index, tx_output) in tx.outputs.iter().enumerate() {
            map.insert(UtxoKey(hash, index), tx_output.to_owned());
        }
    }
    map
}

impl Storage for InMemoryStorage {
    fn get_block(&self, hash: &Hash) -> Option<&Block> {
        self.blocks.get(hash)
    }

    fn get_height(&self, hash: &Hash) -> Option<u64> {
        self.height.get(hash).copied()
    }

    fn get_tip(&self) -> Option<&Block> {
        self.tip.as_ref().and_then(|h| self.blocks.get(h))
    }

    fn insert_block(&mut self, block: Block, hasher: &dyn Hasher) -> Result<(), String> {
        let hash = block.hash(hasher);
        if self.blocks.contains_key(&hash) {
            return Err("Block already exists".into());
        }
        let prev_hash = block.header.prev_hash;

        let new_height = if self.blocks.is_empty() {
            if prev_hash != Hash::zero() {
                return Err("Genesis block must have zero prev_hash".into());
            }
            0
        } else {
            let parent_height = self
                .height
                .get(&prev_hash)
                .copied()
                .ok_or("Missing parent block")?;

            parent_height + 1
        };

        self.blocks.insert(hash, block);
        self.height.insert(hash, new_height);

        if let Some(tip_hash) = &self.tip {
            let tip_height = self.height.get(tip_hash).copied().ok_or("Corrupted tip")?;
            if new_height > tip_height {
                self.tip = Some(hash);
            }
        } else {
            self.tip = Some(hash);
        }

        Ok(())
    }

    fn get_utxo(&self, key: &UtxoKey) -> Option<&TxOutput> {
        self.utxo_map.get(key)
    }

    fn contains_utxo(&self, key: &UtxoKey) -> bool {
        self.utxo_map.contains_key(key)
    }

    fn apply_tx(&mut self, tx: &Tx, hasher: &dyn Hasher) {
        let tx_hash = tx.hash(hasher);
        for input in &tx.inputs {
            self.utxo_map
                .remove(&UtxoKey(input.prev_tx, input.output_index));
        }

        for (i, output) in tx.outputs.iter().enumerate() {
            self.utxo_map.insert(UtxoKey(tx_hash, i), output.clone());
        }
    }
}
