use crate::common::types::{block::Block, tx::Tx};

pub enum NodeEvent {
    Quit,
    NewTransaction(Tx),
    NewBlock(Block),
}
