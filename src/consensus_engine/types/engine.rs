use crate::block::Block;
use crate::transaction::Transaction;
use crate::consensus_engine::traits::mempool::Mempool;
use crate::consensus_engine::traits::miner::Miner;
use crate::consensus_engine::traits::network::Network;
use crate::consensus_engine::traits::storage::Storage;
use crate::consensus_engine::validation::block_validator::BlockValidator;
use crate::consensus_engine::validation::chain_validator::ChainValidator;

pub struct Engine {
    storage: Box<dyn Storage>,
    network: Box<dyn Network>,
    mempool: Box<dyn Mempool>,
    miner: Box<dyn Miner>,
    difficulty: usize,
}

impl Engine {
    pub fn new(
        storage: Box<dyn Storage>,
        network: Box<dyn Network>,
        mempool: Box<dyn Mempool>,
        miner: Box<dyn Miner>,
        difficulty: usize,
    ) -> Self {
        Self { storage, network, mempool, miner, difficulty }
    }

    pub fn add_transaction(&self, tx: Transaction) {
        self.mempool.add_transaction_to_mempool(&tx);
    }

    pub fn receive_block(&mut self, block_incoming: Block) {
        let last_block = self.storage.get_last_block();
        if !BlockValidator::validate(&block_incoming, last_block.as_ref()) {
            return;
        }
        self.storage.save(&block_incoming);
        self.mempool.remove_transactions(&block_incoming.transactions);
        self.network.broadcast_block(&block_incoming);
    }

    pub fn mine_new_block(&mut self) {
        let txs = self.mempool.get_pending_transactions();
        let last_block = self.storage.get_last_block();

        let candidate = match &last_block {
            None => Block::new(0, txs.clone(), String::new()),
            Some(lb) => Block::new(lb.index + 1, txs.clone(), lb.hash.clone()),
        };

        let mined = self.miner.mine(candidate, self.difficulty);

        if !BlockValidator::validate(&mined, last_block.as_ref()) {
            return;
        }

        self.storage.save(&mined);
        self.network.broadcast_block(&mined);
        self.mempool.remove_transactions(&txs);
    }

    pub fn broadcast_current_chain(&self) {
        if let Some(last) = self.storage.get_last_block() {
            let chain = self.storage.get_chain(&last);
            self.network.broadcast_chain(chain);
        }
    }

    pub fn receive_chain(&mut self, block: Block, chain: Vec<Block>) {
        let local_chain = self.storage.get_chain(&block);
        if !ChainValidator::validate(&chain) {
            return;
        }
        if chain.len() > local_chain.len() {
            let new_chain = self.storage.replace_chain(block, chain);
            self.network.broadcast_chain(new_chain);
        }
    }
}
