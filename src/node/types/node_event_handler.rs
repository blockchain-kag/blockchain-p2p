use std::{
    sync::mpsc::{Receiver, Sender},
    thread::{self, JoinHandle},
};

use crate::node::types::node_event::NodeEvent;

pub struct NodeEventHandler {
    input_receiver: Receiver<String>,
    event_sender: Sender<NodeEvent>,
}

impl NodeEventHandler {
    pub fn new(input_receiver: Receiver<String>, event_sender: Sender<NodeEvent>) -> Self {
        Self {
            input_receiver,
            event_sender,
        }
    }

    pub fn run(self) -> JoinHandle<()> {
        thread::spawn(move || {
            while let Ok(input) = self.input_receiver.recv() {
                match input.as_str() {
                    "quit" => {
                        self.event_sender.send(NodeEvent::Quit).unwrap();
                        break;
                    }
                    "help" => {
                        self.event_sender.send(NodeEvent::ListCommands).unwrap();
                    }
                    _ => {}
                }
            }
        })
    }
}
