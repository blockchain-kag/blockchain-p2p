use std::{
    ops::ControlFlow,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender},
    },
    thread::sleep,
};

use crate::{
    common::ports::hasher::Hasher,
    consensus_engine::types::consensus_engine::ConsensusEngine,
    mempool::types::mempool::Mempool,
    node::{ports::storage::Storage, types::node_event::NodeEvent},
};

pub struct Node {
    event_stream: Receiver<NodeEvent>,
    shutdown: Arc<AtomicBool>,
    emmitter: Sender<String>,
    mempool: Mempool,
    storage: Box<dyn Storage>,
    consensus_engine: ConsensusEngine,
    hasher: Box<dyn Hasher>,
}

impl Node {
    pub fn new(
        event_stream: Receiver<NodeEvent>,
        shutdown: Arc<AtomicBool>,
        emmitter: Sender<String>,
        mempool: Mempool,
        storage: Box<dyn Storage>,
        consensus_engine: ConsensusEngine,
        hasher: Box<dyn Hasher>,
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

    pub fn run(&mut self) {
        self.emmitter
            .send(String::from("Node starting..."))
            .unwrap();
        while !self.shutdown.load(Ordering::Relaxed) {
            while let Ok(event) = self.event_stream.try_recv() {
                match self.manage_event(event) {
                    ControlFlow::Continue(_) => continue,
                    ControlFlow::Break(_) => break,
                }
            }

            sleep(std::time::Duration::from_millis(10));
        }
        self.emmitter.send(String::from("Node stoping...")).unwrap();
    }

    fn manage_event(&mut self, event: NodeEvent) -> ControlFlow<()> {
        match event {
            NodeEvent::ListCommands => {
                self.emmitter.send(String::from("quit & help")).unwrap();
            }
            NodeEvent::Quit => {
                self.shutdown.store(true, Ordering::Relaxed);
                return ControlFlow::Break(());
            }
            NodeEvent::NewTransaction(tx) => self.mempool.push(tx),
            NodeEvent::NewBlock(block) => match self.storage.get_tip() {
                Some(prev_block) => {
                    if self.consensus_engine.validate(&prev_block, &block) {
                        self.storage.insert_block(block).unwrap();
                    };
                }
                None => {
                    self.storage.insert_block(block).unwrap();
                }
            },
            NodeEvent::StartMining => {
                let txs = self.mempool.get_first_n(10);
                let last_block = self.storage.get_tip().unwrap();
                self.consensus_engine
                    .start_mining(txs, &last_block, self.hasher.as_ref())
                    .unwrap();
            }
            NodeEvent::PauseMining => self.consensus_engine.pause_mining().unwrap(),
            NodeEvent::ContinueMining => self.consensus_engine.continue_mining().unwrap(),
            NodeEvent::StopMining => self.consensus_engine.stop_mining().unwrap(),
        }
        ControlFlow::Continue(())
    }
}
