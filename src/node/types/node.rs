use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender},
    },
    thread::sleep,
};

use crate::node::types::node_event::NodeEvent;

pub struct Node {
    event_stream: Receiver<NodeEvent>,
    shutdown: Arc<AtomicBool>,
    emmitter: Sender<String>,
}

impl Node {
    pub fn new(
        event_stream: Receiver<NodeEvent>,
        shutdown: Arc<AtomicBool>,
        emmitter: Sender<String>,
    ) -> Self {
        Self {
            event_stream,
            shutdown,
            emmitter,
        }
    }

    pub fn run(&mut self) {
        self.emmitter
            .send(String::from("Node starting..."))
            .unwrap();
        while !self.shutdown.load(Ordering::Relaxed) {
            while let Ok(event) = self.event_stream.try_recv() {
                match event {
                    NodeEvent::ListCommands => {
                        self.emmitter.send(String::from("quit & help")).unwrap();
                    }
                    NodeEvent::Quit => {
                        self.shutdown.store(true, Ordering::Relaxed);
                        break;
                    }
                    NodeEvent::NewTransaction(_tx) => todo!(),
                    NodeEvent::NewBlock(_block) => todo!(),
                }
            }

            sleep(std::time::Duration::from_millis(10));
        }
        self.emmitter.send(String::from("Node stoping...")).unwrap();
    }
}
