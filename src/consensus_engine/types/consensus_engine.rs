use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, SendError, Sender, channel};
use std::thread::{self, JoinHandle, sleep};
use std::time::Instant;

use crate::common::ports::hasher::Hasher;
use crate::common::types::block::Block;
use crate::common::types::tx::{Hash, Tx};
use crate::consensus_engine::ports::block_validator::BlockValidator;
use crate::consensus_engine::ports::miner::{Miner, MinerCommand, MinerEvent};

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

#[derive(Debug, Clone, Default)]
pub enum MinerState {
    Mining,
    Paused,
    #[default]
    Stopped,
}

#[derive(Debug, Clone, Default)]
pub struct MinerData {
    pub state: MinerState,
    pub hash_rate: Option<f64>,
    pub difficulty: usize,
    pub start_instant: Option<Instant>,

    pub current_block_hash: Option<Hash>,
    pub current_nonce: Option<u64>,
    pub attempts: Option<u64>,

    pub last_block_hash: Option<Hash>,
}

impl MinerData {
    pub fn new(difficulty: usize) -> Self {
        MinerData {
            difficulty,
            ..Default::default()
        }
    }
}

pub enum ConsensusEngineEvent {
    Found(Block),
    StateUpdate { worker_id: u64, data: MinerData },
    Error(String),
    BlockValidated(Block, bool),
}
pub enum ConsensusEngineCommand {
    StartMining(Block, usize),
    StopMining,
    PauseMining,
    ResumeMining,
    ValidateBlock(Block, Block),
}

pub struct ConsensusEngine {
    miner: Box<dyn Miner>,
    validator: Box<dyn BlockValidator + Send + Sync>,
    difficulty: usize,
    sources_events_txs: Vec<Sender<ConsensusEngineEvent>>,
    miner_handlers: Vec<Sender<MinerCommand>>,
    miner_events_channel: (Sender<MinerEvent>, Receiver<MinerEvent>),
    sources_commands_channel: (
        Sender<ConsensusEngineCommand>,
        Receiver<ConsensusEngineCommand>,
    ),
}

impl ConsensusEngine {
    pub fn new(
        miner: Box<dyn Miner>,
        validator: Box<dyn BlockValidator>,
        difficulty: usize,
        sources_events_txs: Vec<Sender<ConsensusEngineEvent>>,
    ) -> (Sender<ConsensusEngineCommand>, Self) {
        let miner_events_channel = channel();
        let sources_commands_channel = channel();
        (
            sources_commands_channel.0.clone(),
            Self {
                miner,
                validator,
                difficulty,
                sources_events_txs,
                miner_handlers: vec![],
                miner_events_channel,
                sources_commands_channel,
            },
        )
    }

    pub fn run(mut self) -> Result<JoinHandle<()>, ConsensusEngineError> {
        Ok(thread::spawn(move || {
            loop {
                while let Ok(event) = self.miner_events_channel.1.try_recv() {
                    match event {
                        MinerEvent::Found(block) => {
                            for tx in &self.miner_handlers {
                                let _ = tx.send(MinerCommand::Stop);
                            }
                            for source_event_tx in &self.sources_events_txs {
                                source_event_tx
                                    .send(ConsensusEngineEvent::Found(block.clone()))
                                    .unwrap();
                            }
                        }
                        MinerEvent::StateUpdate { worker_id, data } => {
                            for source_event_tx in &self.sources_events_txs {
                                source_event_tx
                                    .send(ConsensusEngineEvent::StateUpdate {
                                        worker_id,
                                        data: data.clone(),
                                    })
                                    .unwrap();
                            }
                        }
                        MinerEvent::Error(string) => {
                            for source_event_tx in &self.sources_events_txs {
                                source_event_tx
                                    .send(ConsensusEngineEvent::Error(string.clone()))
                                    .unwrap();
                            }
                        }
                        _ => {}
                    }
                }
                while let Ok(command) = self.sources_commands_channel.1.try_recv() {
                    match command {
                        ConsensusEngineCommand::StartMining(block_to_mine, minners) => {
                            for tx in &self.miner_handlers {
                                tx.send(MinerCommand::Stop).unwrap();
                            }
                            if minners > self.miner_handlers.len() {
                                for _ in self.miner_handlers.len()..minners {
                                    self.miner_handlers.push(
                                        self.miner
                                            .spawn(self.miner_events_channel.0.clone())
                                            .unwrap(),
                                    );
                                }
                            }

                            for (i, miner_command_tx) in self.miner_handlers.iter().enumerate() {
                                let mut block = block_to_mine.clone();
                                block.header.nonce = i as u64;

                                miner_command_tx
                                    .send(MinerCommand::Start {
                                        block,
                                        difficulty: self.difficulty,
                                        worker_id: i,
                                        num_workers: minners,
                                    })
                                    .unwrap();
                            }
                        }
                        ConsensusEngineCommand::StopMining => {
                            for miner_command_tx in &self.miner_handlers {
                                miner_command_tx.send(MinerCommand::Stop).unwrap();
                            }
                        }
                        ConsensusEngineCommand::PauseMining => {
                            for miner_command_tx in &self.miner_handlers {
                                miner_command_tx.send(MinerCommand::Pause).unwrap();
                            }
                        }
                        ConsensusEngineCommand::ResumeMining => {
                            for miner_command_tx in &self.miner_handlers {
                                miner_command_tx.send(MinerCommand::Resume).unwrap();
                            }
                        }
                        ConsensusEngineCommand::ValidateBlock(block, prev_block) => {
                            for source_event_tx in &self.sources_events_txs {
                                source_event_tx
                                    .send(ConsensusEngineEvent::BlockValidated(
                                        block.clone(),
                                        self.validator.validate(&prev_block, &block),
                                    ))
                                    .unwrap();
                            }
                        }
                    };
                }
                sleep(std::time::Duration::from_millis(1));
            }
        }))
    }
}
