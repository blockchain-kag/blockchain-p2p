use crate::common::types::{block::Block, tx::Tx};

pub enum NodeEvent {
    ListCommands,
    Quit,
    NewTransaction(Tx),
    NewBlock(Block),
    StartMining,
    PauseMining,
    ContinueMining,
    StopMining,
}
