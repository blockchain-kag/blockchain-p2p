use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::sleep,
};

use crate::{
    consensus_engine::types::consensus_engine::ConsensusEngine,
    events::{ports::event_stream::EventStream, types::node_event::NodeEvent},
    mempool::types::mempool::Mempool,
    node::ports::emmitter::Emmitter,
    storage::ports::storage::Storage,
};

pub struct Node {
    event_stream: Box<dyn EventStream>,
    shutdown: Arc<AtomicBool>,
    emmitter: Arc<Mutex<dyn Emmitter>>,
}

impl Node {
    pub fn new(
        event_stream: Box<dyn EventStream + Send>,
        shutdown: Arc<AtomicBool>,
        emmitter: Arc<Mutex<dyn Emmitter>>,
    ) -> Self {
        Self {
            event_stream,
            shutdown,
            emmitter,
        }
    }

    pub fn run(&mut self) {
        self.emmitter
            .lock()
            .unwrap()
            .emmit(String::from("Node starting...\n"))
            .unwrap();
        while !self.shutdown.load(Ordering::Relaxed) {
            while let Some(event) = self.event_stream.try_recv() {
                match event {
                    NodeEvent::Quit => {
                        self.shutdown.store(true, Ordering::Relaxed);
                        break;
                    }
                    NodeEvent::NewTransaction(tx) => todo!(),
                    NodeEvent::NewBlock(block) => todo!(),
                }
            }

            sleep(std::time::Duration::from_millis(10));
        }
        self.emmitter
            .lock()
            .unwrap()
            .emmit(String::from("\nNode stoping...\n"))
            .unwrap();
    }
}
