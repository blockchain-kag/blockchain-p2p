use crate::events::types::node_event::NodeEvent;

pub trait EventStream {
    fn try_recv(&mut self) -> Option<NodeEvent>;
}
