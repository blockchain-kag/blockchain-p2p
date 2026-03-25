use crate::consensus_engine::block::block::Block;
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

    // Receive a block from network
    pub fn receive_block(&mut self, block_incoming: Block) {
        let last_block_opt = self.storage.get_last_block();

        match last_block_opt {
            None => {
                if !BlockValidator::validate(&block_incoming, None) { return; }
                self.storage.save(&block_incoming);
                self.mempool.remove_transactions(&block_incoming.transactions);
                self.network.broadcast_block(&block_incoming);
            }

            Some(last_block) => {
                if !BlockValidator::validate(&block_incoming, Some(last_block)) { return; }
                self.storage.save(&block_incoming);
                self.mempool.remove_transactions(&block_incoming.transactions);
                self.network.broadcast_block(&block_incoming);
            }
        }
    }

    // Create a block (local mining)
    pub fn mine_new_block(&mut self) {
        let txs = self.mempool.get_pending_transactions();
        let last_block = self.storage.get_last_block();
        let candidate = match last_block {
            None => {
                Block::new(
                    0,
                    txs.clone(),
                    String::new(),
                )
            }

            Some(last_block) => {
                Block::new(
                    last_block.index + 1,
                    txs.clone(),
                    last_block.hash.clone(),
                )
            }
        };

        let mined = self.miner.mine(candidate, self.difficulty);

        if !BlockValidator::validate(&mined, last_block) { return; }

        self.storage.save(&mined);
        self.network.broadcast_block(&mined);
        self.mempool.remove_transactions(&txs);
    }

    // Receive a chain from network
    pub fn receive_chain(&mut self, chain: Vec<Block>) {
        let local_chain = self.storage.get_chain();

        if !ChainValidator::validate(&chain) { return; }

        if chain.len() > local_chain.len() {
            self.storage.replace_chain(&chain);
            self.network.broadcast_chain(&chain);
        }
    }

}