use std::sync::mpsc::Receiver;

use crate::events::{ports::event_stream::EventStream, types::node_event::NodeEvent};

pub struct FifoEventStream {
    rx: Receiver<NodeEvent>,
}

impl FifoEventStream {
    pub fn new(rx: Receiver<NodeEvent>) -> Self {
        Self { rx }
    }
}

impl EventStream for FifoEventStream {
    fn try_recv(&mut self) -> Option<NodeEvent> {
        self.rx.try_recv().ok()
    }
}
