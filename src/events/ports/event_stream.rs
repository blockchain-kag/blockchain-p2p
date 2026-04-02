use crate::events::types::node_event::NodeEvent;

pub trait EventSource {
    fn try_recv(&mut self) -> Option<NodeEvent>;
}
