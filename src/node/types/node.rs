use std::{
    collections::HashMap,
    ops::ControlFlow,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender},
    },
    thread::{self, JoinHandle},
};

use crate::{
    common::{
        ports::hasher::Hasher,
        types::{
            tx::{Tx, TxInput, TxOutput},
            wallet::Wallet,
        },
    },
    consensus_engine::types::consensus_engine::ConsensusEngine,
    mempool::types::mempool::Mempool,
    node::{
        ports::storage::{Storage, UtxoKey},
        types::node_command::NodeCommand,
    },
};

pub struct Node {
    event_stream: Receiver<NodeCommand>,
    shutdown: Arc<AtomicBool>,
    emmitter: Sender<String>,
    mempool: Mempool,
    storage: Box<dyn Storage>,
    consensus_engine: ConsensusEngine,
    hasher: Arc<dyn Hasher>,
    wallet: Box<dyn Wallet>,
}

const MAX_TX_PER_BLOCK: usize = 10;
const BLOCK_MINERS: usize = 5;

impl Node {
    pub fn new(
        event_stream: Receiver<NodeCommand>,
        shutdown: Arc<AtomicBool>,
        emmitter: Sender<String>,
        mempool: Mempool,
        storage: Box<dyn Storage>,
        consensus_engine: ConsensusEngine,
        hasher: Arc<dyn Hasher>,
        wallet: Box<dyn Wallet>,
    ) -> Self {
        Self {
            event_stream,
            shutdown,
            emmitter,
            mempool,
            storage,
            consensus_engine,
            hasher,
            wallet,
        }
    }

    pub fn run(mut self) -> JoinHandle<()> {
        thread::spawn(move || {
            self.emmitter
                .send(String::from("Node starting..."))
                .unwrap();
            while let Ok(event) = self.event_stream.recv() {
                match self.manage_event(event) {
                    ControlFlow::Continue(_) => continue,
                    ControlFlow::Break(_) => break,
                }
            }
            self.emmitter.send(String::from("Node stoping...")).unwrap();
        })
    }

    fn manage_event(&mut self, event: NodeCommand) -> ControlFlow<()> {
        let utxo_map = self.storage.get_utxo_map();
        match event {
            NodeCommand::Help => {
                let comment: String = todo!("Write help comment");
                self.emmitter.send(String::from(comment)).unwrap();
            }
            NodeCommand::Quit => {
                self.shutdown.store(true, Ordering::Relaxed);
                return ControlFlow::Break(());
            }
            NodeCommand::SaveTransaction(tx) => {
                self.mempool.push(tx, utxo_map);
                todo!("Do network broadcast of txs")
            }
            NodeCommand::SaveBlock(block) => {
                match self.storage.get_tip() {
                    Some(prev_block) => {
                        if self.consensus_engine.is_block_valid(prev_block, &block) {
                            self.storage
                                .insert_block(block, self.hasher.as_ref())
                                .unwrap();
                            self.restart_mining();
                        };
                    }
                    None => {
                        self.emmitter
                            .send("No genesis block found".to_string())
                            .unwrap();
                    }
                }
                todo!("Do network broadcast of block");
            }
            NodeCommand::StartMining(miners) => {
                let txs = self.mempool.get_first_n(MAX_TX_PER_BLOCK);
                let last_block = self.storage.get_tip().unwrap();
                self.consensus_engine
                    .start_mining(txs, last_block, self.hasher.as_ref(), miners)
                    .unwrap();
            }
            NodeCommand::PauseMining => self.consensus_engine.pause_mining().unwrap(),
            NodeCommand::ResumeMining => self.consensus_engine.resume_mining().unwrap(),
            NodeCommand::StopMining => self.consensus_engine.stop_mining().unwrap(),
            NodeCommand::Transfer(transfers, fee) => {
                let total_output: u64 = transfers.iter().map(|(_, amt)| amt).sum();
                let (unsigned_inputs, total_input) =
                    select_utxos(total_output + fee, utxo_map).unwrap();
                let mut outputs: Vec<TxOutput> = transfers
                    .iter()
                    .map(|transfer| TxOutput {
                        amount: transfer.1,
                        recipient: transfer.0.clone().into(),
                    })
                    .collect();
                let change = total_input - total_output - fee;
                if change > 0 {
                    outputs.push(TxOutput {
                        amount: change,
                        recipient: self.wallet.change_address(),
                    });
                }
                let tx = Tx {
                    inputs: unsigned_inputs,
                    outputs,
                }
                .sign(self.hasher.as_ref(), self.wallet.as_ref());
                self.mempool.push(tx.clone(), self.storage.get_utxo_map());
                todo!("Networking of tx && change management")
            }
            NodeCommand::StartSyncing => {
                todo!("Netowrk syncing")
            }
        }
        ControlFlow::Continue(())
    }

    fn restart_mining(&mut self) {
        self.consensus_engine.stop_mining().unwrap();
        self.consensus_engine
            .start_mining(
                self.mempool.get_first_n(MAX_TX_PER_BLOCK),
                self.storage.get_tip().unwrap(),
                self.hasher.as_ref(),
                BLOCK_MINERS,
            )
            .unwrap();
    }
}

fn select_utxos(amount: u64, utxo_map: &HashMap<UtxoKey, TxOutput>) -> Option<(Vec<TxInput>, u64)> {
    let mut utxos: Vec<(&UtxoKey, &TxOutput)> = utxo_map.iter().collect();
    utxos.sort_by(|a, b| b.1.amount.cmp(&a.1.amount));

    let mut selected = Vec::new();
    let mut total = 0u64;

    for (key, utxo) in utxos {
        selected.push(TxInput {
            prev_tx: key.0,
            output_index: key.1,
            signature: vec![],
            pubkey: vec![],
        });

        total = total.checked_add(utxo.amount)?;

        if total >= amount {
            return Some((selected, total));
        }
    }
    None
}
