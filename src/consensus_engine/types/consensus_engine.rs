use std::collections::VecDeque;
use std::sync::mpsc::SendError;

use crate::common::ports::hasher::Hasher;
use crate::common::types::block::Block;
use crate::common::types::tx::Tx;
use crate::consensus_engine::ports::block_validator::BlockValidator;
use crate::consensus_engine::ports::miner::{Miner, MinerCommand, MinerHandle};

#[derive(Debug)]
pub enum ConsensusEngineError {
    MinerError(String),
}

impl From<String> for ConsensusEngineError {
    fn from(value: String) -> Self {
        ConsensusEngineError::MinerError(value)
    }
}

impl<T> From<SendError<T>> for ConsensusEngineError {
    fn from(value: SendError<T>) -> Self {
        ConsensusEngineError::MinerError(format!("Failed to send miner command: {value:?}"))
    }
}

pub struct ConsensusEngine {
    miner: Box<dyn Miner>,
    miner_handlers: Vec<MinerHandle>,
    validator: Box<dyn BlockValidator + Send + Sync>,
    difficulty: usize,
}

impl ConsensusEngine {
    pub fn new(
        miner: Box<dyn Miner>,
        validator: Box<dyn BlockValidator>,
        difficulty: usize,
    ) -> Self {
        Self {
            miner,
            miner_handlers: vec![],
            validator,
            difficulty,
        }
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
    ) -> Result<(), ConsensusEngineError> {
        let block = Block::new(0, last_block.header.hash(hasher), 0, txs, hasher);

        if miners > self.miner_handlers.len() {
            for _ in self.miner_handlers.len()..miners {
                let handler = self.miner.spawn()?;
                self.miner_handlers.push(handler);
            }
        }

        for (i, handler) in self.miner_handlers.iter().enumerate() {
            let mut block = block.clone();
            block.header.nonce = i as u64;

            handler.sender.send(MinerCommand::Start {
                block,
                difficulty: self.difficulty,
                worker_id: i,
                num_workers: miners,
            })?;
        }
        Ok(())
    }

    pub fn stop_mining(&self) -> Result<(), ConsensusEngineError> {
        for handler in self.miner_handlers.iter() {
            handler.sender.send(MinerCommand::Stop)?;
        }
        Ok(())
    }
    pub fn pause_mining(&self) -> Result<(), ConsensusEngineError> {
        for handler in self.miner_handlers.iter() {
            handler.sender.send(MinerCommand::Pause)?;
        }
        Ok(())
    }
    pub fn resume_mining(&self) -> Result<(), ConsensusEngineError> {
        for handler in self.miner_handlers.iter() {
            handler.sender.send(MinerCommand::Resume)?;
        }
        Ok(())
    }
}
