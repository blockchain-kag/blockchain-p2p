use crate::common::types::{block::Block, tx::Tx};

pub enum NodeCommand {
    Quit,
    Help,
    Transfer(Vec<(String, u64)>, u64),
    SaveTransaction(Tx),
    SaveBlock(Block),
    StartMining(usize),
    StopMining,
    ResumeMining,
    PauseMining,
    StartSyncing,
}
