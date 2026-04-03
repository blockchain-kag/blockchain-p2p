use std::{
    sync::{Arc, Mutex, mpsc::Sender},
    thread,
};

use crate::{
    events::{ports::event_producer::EventProvider, types::node_event::NodeEvent},
    node::ports::emmitter::Emmitter,
};
use std::io::{self, BufRead};

pub struct UserEventProvider {
    emmitter: Arc<Mutex<dyn Emmitter>>,
}

impl UserEventProvider {
    pub fn new(emmitter: Arc<Mutex<dyn Emmitter>>) -> Self {
        Self { emmitter }
    }
}

impl EventProvider for UserEventProvider {
    fn spawn(self, tx: Sender<NodeEvent>) {
        thread::spawn(move || {
            let stdin = io::stdin();
            let mut lines = stdin.lock().lines();
            loop {
                self.emmitter
                    .lock()
                    .unwrap()
                    .emmit(String::from("> "))
                    .unwrap();

                if let Some(Ok(line)) = lines.next() {
                    let input = line.trim();
                    let event = match input {
                        "quit" => Some(NodeEvent::Quit),
                        _ => None,
                    };
                    if let Some(event) = event {
                        if tx.send(event).is_err() {
                            break;
                        }
                    }
                } else {
                    thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        });
    }
}
