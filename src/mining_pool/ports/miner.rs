use std::sync::mpsc::{Receiver, Sender};

use crate::{common::types::block::Block, mining_pool::types::miner_data::MinerData};

#[derive(Clone)]
pub enum MinerCommand {
    Start {
        block: Block,
        difficulty: usize,
        worker_id: usize,
        num_workers: usize,
    },
    Pause,
    Resume,
    Stop,
    PollData,
}

pub enum MinerEvent {
    Found(Block),
    StateUpdate { worker_id: u64, data: MinerData },
    Started,
    Resumed,
    Paused,
    Stopped,
    Error(String),
}

pub struct MinerHandle {
    pub sender: Sender<MinerCommand>,
    pub receiver: Receiver<MinerEvent>,
}

impl MinerHandle {
    pub fn new(sender: Sender<MinerCommand>, receiver: Receiver<MinerEvent>) -> Self {
        Self { sender, receiver }
    }
}

pub trait Miner: Send + Sync {
    fn spawn(&self, tx: Sender<MinerEvent>) -> Result<Sender<MinerCommand>, String>;
}
