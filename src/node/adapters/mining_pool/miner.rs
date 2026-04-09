use std::{
    sync::{
        Arc,
        mpsc::{Sender, channel},
    },
    thread::{self, sleep},
    time::{Duration, Instant},
};

use tokio::time::error::Elapsed;

use crate::{
    common::{
        ports::hasher::Hasher,
        types::{block::Block, tx::Hash},
    },
    mining_pool::{
        ports::miner::{Miner, MinerCommand, MinerEvent},
        types::miner_data::{MinerData, MinerState},
    },
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
    fn spawn(&self, tx: Sender<MinerEvent>) -> Result<Sender<MinerCommand>, String> {
        let (in_tx, in_rx) = channel::<MinerCommand>();
        let hasher = self.hasher.clone();

        thread::spawn(move || {
            let mut start_instant = Some(Instant::now());
            let mut accumulated_mining_time = Duration::ZERO;
            let mut mine = false;
            let mut mined_block: Option<Block> = None;
            let mut mining_difficulty: Option<usize> = None;
            let mut received_worker_id = 0;
            let mut workers_amount = 1;
            let mut miner_data = MinerData::default();
            let mut attempts = 0;
            loop {
                while let Ok(command) = in_rx.try_recv() {
                    match command {
                        MinerCommand::Start {
                            block,
                            difficulty,
                            worker_id,
                            num_workers,
                        } => {
                            mine = true;
                            received_worker_id = worker_id as u64;
                            workers_amount = num_workers;
                            let nonce = received_worker_id;
                            mined_block = Some({
                                let mut b = block;
                                b.header.nonce = nonce;
                                b
                            });
                            start_instant = Some(Instant::now());
                            accumulated_mining_time = Duration::ZERO;
                            mining_difficulty = Some(difficulty);
                            miner_data = MinerData::new(difficulty);
                            miner_data.elapsed = accumulated_mining_time;
                            miner_data.attempts = Some(0);
                            miner_data.current_nonce = Some(nonce);
                            miner_data.current_block_hash = None;
                            tx.send(MinerEvent::Started).unwrap();
                            miner_data.state = MinerState::Mining;
                            tx.send(MinerEvent::StateUpdate {
                                worker_id: received_worker_id,
                                data: miner_data.clone(),
                            })
                            .unwrap();
                        }
                        MinerCommand::Pause => {
                            mine = false;
                            tx.send(MinerEvent::Paused).unwrap();
                            if let Some(s) = start_instant {
                                accumulated_mining_time += s.elapsed();
                            }
                            start_instant = None;
                            miner_data.state = MinerState::Paused;
                            tx.send(MinerEvent::StateUpdate {
                                worker_id: received_worker_id,
                                data: miner_data.clone(),
                            })
                            .unwrap();
                        }
                        MinerCommand::Resume => {
                            mine = true;
                            tx.send(MinerEvent::Resumed).unwrap();
                            miner_data.state = MinerState::Mining;
                            start_instant = Some(Instant::now());
                            tx.send(MinerEvent::StateUpdate {
                                worker_id: received_worker_id,
                                data: miner_data.clone(),
                            })
                            .unwrap();
                        }
                        MinerCommand::Stop => {
                            mine = false;
                            mined_block = None;
                            mining_difficulty = None;
                            tx.send(MinerEvent::Stopped).unwrap();
                            miner_data.state = MinerState::Stopped;
                            if let Some(s) = start_instant {
                                accumulated_mining_time += s.elapsed();
                            }
                            start_instant = None;
                            tx.send(MinerEvent::StateUpdate {
                                worker_id: received_worker_id,
                                data: miner_data.clone(),
                            })
                            .unwrap();
                            miner_data = MinerData::default();
                        }
                        MinerCommand::PollData => {
                            if let Some(s) = start_instant {
                                miner_data.elapsed = accumulated_mining_time + s.elapsed();
                            } else {
                                miner_data.elapsed = accumulated_mining_time;
                            }
                            tx.send(MinerEvent::StateUpdate {
                                worker_id: received_worker_id,
                                data: miner_data.clone(),
                            })
                            .unwrap();
                        }
                    }
                }

                if mine && let (Some(block), Some(diff)) = (&mut mined_block, mining_difficulty) {
                    let hash: Hash = block.hash(hasher.clone());
                    attempts += 1;
                    miner_data.attempts = Some(attempts);
                    if let Some(s) = start_instant {
                        miner_data.elapsed = accumulated_mining_time + s.elapsed();
                    } else {
                        miner_data.elapsed = accumulated_mining_time;
                    }
                    miner_data.current_nonce = Some(block.header.nonce);
                    miner_data.current_block_hash = Some(hash);
                    let duration_in_secs = accumulated_mining_time.as_secs_f64();
                    if duration_in_secs > 0.0 {
                        miner_data.hash_rate = Some(attempts as f64 / duration_in_secs)
                    }
                    tx.send(MinerEvent::StateUpdate {
                        worker_id: received_worker_id,
                        data: miner_data.clone(),
                    })
                    .unwrap();
                    if hash.0.iter().take(diff).all(|&b| b == 0) {
                        tx.send(MinerEvent::Found(block.clone())).unwrap();
                        miner_data.state = MinerState::Stopped;
                        tx.send(MinerEvent::StateUpdate {
                            worker_id: received_worker_id,
                            data: miner_data.clone(),
                        })
                        .unwrap();
                        mined_block = None;
                        mining_difficulty = None;
                        mine = false;
                    } else {
                        block.header.nonce += workers_amount as u64;
                    }
                } else {
                    sleep(Duration::from_millis(10));
                    continue;
                }
            }
        });

        Ok(in_tx)
    }
}
