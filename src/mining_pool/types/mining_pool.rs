use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::{self, sleep};
use std::time::Duration;

use crate::common::types::block::Block;
use crate::mining_pool::ports::miner::{Miner, MinerCommand, MinerEvent};
use crate::mining_pool::types::miner_data::MinerData;
use crate::mining_pool::types::mining_pool_error::MiningPoolError;

#[derive(Clone)]
pub enum MiningPoolEvent {
    Found(Block),
    StateUpdate { worker_id: u64, data: MinerData },
    Error(String),
    BlockValidated(Block, bool),
}
pub enum MiningPoolCommand {
    StartMining(Block, usize),
    StopMining,
    PauseMining,
    ResumeMining,
}

pub struct MiningPool {
    miner: Box<dyn Miner>,
    difficulty: usize,
    event_subscribers: Vec<Sender<MiningPoolEvent>>,
    worker_senders: Vec<Sender<MinerCommand>>,
    pool_event_sender: Sender<MinerEvent>,
    pool_event_receiver: Receiver<MinerEvent>,
    pool_command_sender: Sender<MiningPoolCommand>,
    pool_command_receiver: Receiver<MiningPoolCommand>,
}

impl MiningPool {
    pub fn new(
        miner: Box<dyn Miner>,
        difficulty: usize,
        event_subscribers: Vec<Sender<MiningPoolEvent>>,
    ) -> (Sender<MiningPoolCommand>, Self) {
        let (pool_event_sender, pool_event_receiver) = channel();
        let (pool_command_sender, pool_command_receiver) = channel();
        (
            pool_command_sender.clone(),
            Self {
                miner,
                difficulty,
                event_subscribers,
                worker_senders: vec![],
                pool_event_sender,
                pool_event_receiver,
                pool_command_sender,
                pool_command_receiver,
            },
        )
    }

    pub fn run(mut self) -> Result<Sender<MiningPoolCommand>, MiningPoolError> {
        let in_tx = self.pool_command_sender;
        thread::spawn(move || {
            loop {
                while let Ok(event) = self.pool_event_receiver.try_recv() {
                    manage_event(event, &self.worker_senders, &self.event_subscribers).unwrap();
                }
                while let Ok(command) = self.pool_command_receiver.try_recv() {
                    manage_command(
                        command,
                        &mut self.worker_senders,
                        &self.pool_event_sender,
                        self.difficulty,
                        self.miner.as_ref(),
                    )
                    .unwrap();
                }
                sleep(Duration::from_millis(1));
            }
        });
        Ok(in_tx)
    }
}

fn broadcast<T: Clone>(
    event: T,
    pool_event_senders: &Vec<Sender<T>>,
) -> Result<(), MiningPoolError> {
    for sender in pool_event_senders {
        sender.send(event.clone())?;
    }
    Ok(())
}

fn manage_command(
    command: MiningPoolCommand,
    worker_senders: &mut Vec<Sender<MinerCommand>>,
    new_worker_sender: &Sender<MinerEvent>,
    difficulty: usize,
    miner_spawn: &dyn Miner,
) -> Result<(), MiningPoolError> {
    match command {
        MiningPoolCommand::StartMining(block_to_mine, minners) => {
            broadcast(MinerCommand::Stop, worker_senders)?;
            if minners > worker_senders.len() {
                for _ in worker_senders.len()..minners {
                    worker_senders.push(miner_spawn.spawn(new_worker_sender.clone())?)
                }
            }

            for (i, miner_command_tx) in worker_senders.iter().enumerate() {
                let mut block = block_to_mine.clone();
                block.header.nonce = i as u64;

                miner_command_tx.send(MinerCommand::Start {
                    block,
                    difficulty,
                    worker_id: i,
                    num_workers: minners,
                })?
            }
        }
        MiningPoolCommand::StopMining => broadcast(MinerCommand::Stop, worker_senders)?,
        MiningPoolCommand::PauseMining => broadcast(MinerCommand::Pause, worker_senders)?,
        MiningPoolCommand::ResumeMining => broadcast(MinerCommand::Resume, worker_senders)?,
    }
    Ok(())
}

fn manage_event(
    event: MinerEvent,
    worker_senders: &Vec<Sender<MinerCommand>>,
    pool_event_senders: &Vec<Sender<MiningPoolEvent>>,
) -> Result<(), MiningPoolError> {
    match event {
        MinerEvent::Found(block) => {
            broadcast(MinerCommand::Stop, worker_senders)?;
            broadcast(MiningPoolEvent::Found(block.clone()), pool_event_senders)?
        }
        MinerEvent::StateUpdate { worker_id, data } => broadcast(
            MiningPoolEvent::StateUpdate { worker_id, data },
            pool_event_senders,
        )?,
        MinerEvent::Error(string) => {
            broadcast(MiningPoolEvent::Error(string.clone()), pool_event_senders)?
        }
        _ => {}
    }
    Ok(())
}
