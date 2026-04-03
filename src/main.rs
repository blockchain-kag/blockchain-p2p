use std::sync::{Arc, Mutex, atomic::AtomicBool, mpsc::channel};

use blockchain_p2p::{
    events::{ports::event_producer::EventProvider, types::node_event::NodeEvent},
    node::{
        adapters::{
            emmitter::console_emmitter::ConsoleEmmitter,
            events::{
                producer::user_event_provider::UserEventProvider,
                streams::fifo_event_stream::FifoEventStream,
            },
        },
        types::node::Node,
    },
};

fn main() {
    let (tx, rx) = channel::<NodeEvent>();
    let node_stream = FifoEventStream::new(rx);
    let emmitter = Arc::new(Mutex::new(ConsoleEmmitter::default()));
    let mut node = Node::new(
        Box::new(node_stream),
        Arc::new(AtomicBool::new(false)),
        emmitter.clone(),
    );
    let user_input_stream = UserEventProvider::new(emmitter.clone());
    user_input_stream.spawn(tx.clone());
    node.run();
}
