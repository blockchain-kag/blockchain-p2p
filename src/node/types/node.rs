use std::{
    ops::ControlFlow,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender},
    },
    thread::{self, JoinHandle},
};

use crate::{
    common::ports::hasher::Hasher,
    consensus_engine::types::consensus_engine::ConsensusEngine,
    mempool::types::mempool::Mempool,
    node::{ports::storage::Storage, types::node_command::NodeCommand},
};

pub struct Node {
    event_stream: Receiver<NodeCommand>,
    shutdown: Arc<AtomicBool>,
    emmitter: Sender<String>,
    mempool: Mempool,
    storage: Box<dyn Storage>,
    consensus_engine: ConsensusEngine,
    hasher: Arc<dyn Hasher>,
}

const MAX_TX_PER_BLOCK: usize = 10;

impl Node {
    pub fn new(
        event_stream: Receiver<NodeCommand>,
        shutdown: Arc<AtomicBool>,
        emmitter: Sender<String>,
        mempool: Mempool,
        storage: Box<dyn Storage>,
        consensus_engine: ConsensusEngine,
        hasher: Arc<dyn Hasher>,
    ) -> Self {
        Self {
            event_stream,
            shutdown,
            emmitter,
            mempool,
            storage,
            consensus_engine,
            hasher,
        }
    }

    pub fn run(mut self) -> JoinHandle<()> {
        thread::spawn(move || {
            self.emmitter
                .send(String::from("Node starting..."))
                .unwrap();
            while let Ok(event) = self.event_stream.recv() {
                match self.manage_event(event) {
                    ControlFlow::Continue(_) => continue,
                    ControlFlow::Break(_) => break,
                }
            }
            self.emmitter.send(String::from("Node stoping...")).unwrap();
        })
    }

    fn manage_event(&mut self, event: NodeCommand) -> ControlFlow<()> {
        match event {
            NodeCommand::Help => {
                self.emmitter.send(String::from("quit & help")).unwrap();
            }
            NodeCommand::Quit => {
                self.shutdown.store(true, Ordering::Relaxed);
                return ControlFlow::Break(());
            }
            NodeCommand::SaveTransaction(tx) => self.mempool.push(tx),
            NodeCommand::SaveBlock(block) => match self.storage.get_tip() {
                Some(prev_block) => {
                    if self.consensus_engine.validate(prev_block, &block) {
                        self.storage.insert_block(block, &*self.hasher).unwrap();
                    };
                }
                None => {
                    self.storage.insert_block(block, &*self.hasher).unwrap();
                }
            },
            NodeCommand::StartMining(miners) => {
                let txs = self.mempool.get_first_n(MAX_TX_PER_BLOCK);
                let last_block = self.storage.get_tip().unwrap();
                self.consensus_engine
                    .start_mining(txs, last_block, self.hasher.as_ref(), miners)
                    .unwrap();
            }
            NodeCommand::PauseMining => self.consensus_engine.pause_mining().unwrap(),
            NodeCommand::ResumeMining => self.consensus_engine.resume_mining().unwrap(),
            NodeCommand::StopMining => self.consensus_engine.stop_mining().unwrap(),
            NodeCommand::StartSyncing => todo!(),
            NodeCommand::Transfer(_, _) => {
                todo!();
            }
        }
        ControlFlow::Continue(())
    }
}
