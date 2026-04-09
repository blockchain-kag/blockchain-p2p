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
        ports::{crypto::Crypto, hasher::Hasher},
        types::{
            block::Block,
            tx::{Tx, TxInput, TxOutput},
            wallet::Wallet,
        },
    },
    mempool::types::mempool::Mempool,
    mining_pool::types::mining_pool::MiningPoolCommand,
    node::{
        ports::storage::{Storage, UtxoKey},
        types::node_command::NodeCommand,
    },
    validator::ports::block_validator::BlockValidator,
};

pub struct Node {
    event_stream: Receiver<NodeCommand>,
    shutdown: Arc<AtomicBool>,
    emmitter: Sender<String>,
    mempool: Mempool,
    storage: Box<dyn Storage>,
    mining_pool_sender: Sender<MiningPoolCommand>,
    block_validator: Box<dyn BlockValidator>,
    hasher: Arc<dyn Hasher>,
    crypto: Arc<dyn Crypto>,
    wallet: Wallet,
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
        mining_pool_sender: Sender<MiningPoolCommand>,
        block_validator: Box<dyn BlockValidator>,
        hasher: Arc<dyn Hasher>,
        crypto: Arc<dyn Crypto>,
    ) -> Self {
        let wallet = Wallet::new(crypto.clone());
        Self {
            event_stream,
            shutdown,
            emmitter,
            mempool,
            storage,
            mining_pool_sender,
            block_validator,
            hasher,
            crypto,
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
                let comment: String = "Missing command".to_string();
                self.emmitter.send(comment).unwrap();
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
                        if self.block_validator.validate(
                            prev_block,
                            &block,
                            self.storage.get_utxo_map(),
                        ) {
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
                let txs = self.mempool.peek_first_n(MAX_TX_PER_BLOCK);
                let last_block = self.storage.get_tip().unwrap();
                let block = Block::new(
                    last_block.header.height + 1,
                    last_block.hash(self.hasher.as_ref()),
                    0,
                    Vec::from(txs),
                    self.hasher.as_ref(),
                );
                self.mining_pool_sender
                    .send(MiningPoolCommand::StartMining(block, miners))
                    .unwrap();
            }
            NodeCommand::PauseMining => {
                self.mining_pool_sender
                    .send(MiningPoolCommand::PauseMining)
                    .unwrap();
            }
            NodeCommand::ResumeMining => {
                self.mining_pool_sender
                    .send(MiningPoolCommand::ResumeMining)
                    .unwrap();
            }
            NodeCommand::StopMining => {
                self.mining_pool_sender
                    .send(MiningPoolCommand::StopMining)
                    .unwrap();
            }
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
                .sign(self.hasher.as_ref(), self.crypto.as_ref());
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
        self.mining_pool_sender
            .send(MiningPoolCommand::StopMining)
            .unwrap();
        let prev_block = self.storage.get_tip().unwrap();
        let txs = self.mempool.peek_first_n(MAX_TX_PER_BLOCK);
        let block = Block::new(
            prev_block.header.height,
            prev_block.hash(self.hasher.as_ref()),
            0,
            Vec::from(txs),
            self.hasher.as_ref(),
        );
        self.mining_pool_sender
            .send(MiningPoolCommand::StartMining(block, BLOCK_MINERS))
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
