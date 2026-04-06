use std::collections::{HashMap, VecDeque};

use crate::common::ports::hasher::Hasher;
use crate::common::types::block::Block;
use crate::common::types::tx::{Tx, TxOutput};
use crate::consensus_engine::ports::block_validator::BlockValidator;
use crate::consensus_engine::ports::miner::{Miner, MinerCommand, MinerHandle};
use crate::node::ports::storage::UtxoKey;

pub struct ConsensusEngine {
    miner: Box<dyn Miner + Send + Sync>,
    miner_handlers: Vec<MinerHandle>,
    validator: Box<dyn BlockValidator + Send + Sync>,
    difficulty: usize,
}

impl ConsensusEngine {
    pub fn new(
        miner: Box<dyn Miner + Send + Sync>,
        validator: Box<dyn BlockValidator + Send + Sync>,
        difficulty: usize,
    ) -> Self {
        Self {
            miner,
            miner_handlers: vec![],
            validator,
            difficulty,
        }
    }

    pub fn is_tx_valid(&self, tx: &Tx, utxo_map: HashMap<UtxoKey, TxOutput>) -> bool {
        todo!()
    }

    pub fn is_block_valid(&self, prev_block: &Block, candidate_block: &Block) -> bool {
        self.validator.validate(prev_block, candidate_block)
    }

    pub fn start_mining(
        &mut self,
        txs: VecDeque<Tx>,
        last_block: &Block,
        hasher: &dyn Hasher,
        miners: usize,
    ) -> Result<(), ()> {
        let block = Block::new(0, last_block.header.hash(hasher), 0, txs, hasher);

        for _ in 0..(self.miner_handlers.len() - miners) {
            let handler = self.miner.spawn().unwrap();
            self.miner_handlers.push(handler);
        }

        for (i, handler) in self.miner_handlers.iter().enumerate() {
            let mut block = block.clone();
            block.header.nonce = i as u64;

            handler
                .sender
                .send(MinerCommand::Start {
                    block,
                    difficulty: self.difficulty,
                    worker_id: i,
                    num_workers: miners,
                })
                .unwrap();
        }
        Ok(())
    }

    pub fn stop_mining(&self) -> Result<(), ()> {
        for handler in self.miner_handlers.iter() {
            handler.sender.send(MinerCommand::Stop).unwrap();
        }
        Ok(())
    }
    pub fn pause_mining(&self) -> Result<(), ()> {
        for handler in self.miner_handlers.iter() {
            handler.sender.send(MinerCommand::Pause).unwrap();
        }
        Ok(())
    }
    pub fn resume_mining(&self) -> Result<(), ()> {
        for handler in self.miner_handlers.iter() {
            handler.sender.send(MinerCommand::Resume).unwrap();
        }
        Ok(())
    }
}
