use std::sync::mpsc::Sender;

use crate::events::types::node_event::NodeEvent;

pub trait EventProvider {
    fn spawn(self, tx: Sender<NodeEvent>);
}
