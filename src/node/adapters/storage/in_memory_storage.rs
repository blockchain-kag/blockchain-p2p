use std::collections::HashMap;

use crate::{
    common::{
        ports::hasher::Hasher,
        types::{block::Block, tx::Hash},
    },
    node::ports::storage::Storage,
};

#[derive(Default)]
pub struct InMemoryStorage {
    height: HashMap<Hash, u64>,
    blocks: HashMap<Hash, Block>,
    tip: Option<Hash>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
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
}
