use std::sync::mpsc::{Receiver, Sender};

use crate::common::types::block::Block;

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
}

pub enum MinerEvent {
    Found(Block),
    Progress { nonce: u64 },
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

pub trait Miner {
    fn spawn(&self) -> Result<MinerHandle, String>;
}
