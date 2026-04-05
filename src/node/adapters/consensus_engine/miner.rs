use std::{
    sync::{Arc, mpsc::channel},
    thread::{self, sleep},
    time::Duration,
};

use crate::{
    common::{
        ports::hasher::Hasher,
        types::{block::Block, tx::Hash},
    },
    consensus_engine::ports::miner::{Miner, MinerCommand, MinerEvent, MinerHandle},
};

pub struct CpuMiner {
    hasher: Arc<dyn Hasher>,
}

impl CpuMiner {
    pub fn new(hasher: Arc<dyn Hasher>) -> Self {
        Self { hasher }
    }
}

impl Miner for CpuMiner {
    fn spawn(&self) -> Result<MinerHandle, String> {
        let (in_tx, in_rx) = channel::<MinerCommand>();
        let (out_tx, out_rx) = channel::<MinerEvent>();
        let hasher = self.hasher.clone();

        thread::spawn(move || {
            let mut mine = false;
            let mut mined_block: Option<Block> = None;
            let mut mining_difficulty: Option<usize> = None;
            loop {
                while let Ok(command) = in_rx.try_recv() {
                    match command {
                        MinerCommand::Start(block, difficulty) => {
                            mine = true;
                            mined_block = Some(block);
                            mining_difficulty = Some(difficulty);
                            out_tx.send(MinerEvent::Started).unwrap();
                        }
                        MinerCommand::Pause => {
                            mine = false;
                            out_tx.send(MinerEvent::Paused).unwrap();
                        }
                        MinerCommand::Resume => {
                            mine = true;
                            out_tx.send(MinerEvent::Resumed).unwrap();
                        }
                        MinerCommand::Stop => {
                            mine = false;
                            mined_block = None;
                            mining_difficulty = None;
                            out_tx.send(MinerEvent::Stopped).unwrap();
                        }
                    }
                }

                if mine && let (Some(block), Some(diff)) = (&mut mined_block, mining_difficulty) {
                    block.header.nonce += 1;
                    let hash: Hash = block.hash(hasher.as_ref());
                    if hash.0.iter().take(diff).all(|&b| b == 0) {
                        out_tx.send(MinerEvent::Found(block.clone())).unwrap();
                        mined_block = None;
                        mining_difficulty = None;
                        mine = false;
                    }
                } else {
                    sleep(Duration::from_millis(10));
                    continue;
                }
            }
        });

        Ok(MinerHandle::new(in_tx, out_rx))
    }
}
