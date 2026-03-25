use crate::consensus_engine::block::block::Block;
use crate::consensus_engine::transaction::transaction::Transaction;
use crate::consensus_engine::traits::hasher::Sha256Hasher;

pub struct BlockFactory;

impl BlockFactory {
    pub fn genesis() -> Block {
        Block::new(
            0,
            vec![],
            "".to_string()
        )
    }

    // fixme
    pub fn valid_after(previous: &Block) -> Block {
        todo!()
    }


    pub fn invalid_pow(previous: &Block) -> Block {
        let mut b = Self::valid_after(previous);
        b.hash = "123456".to_string();
        b
    }


    pub fn invalid_previous_hash(previous: &Block) -> Block {
        Block::new(
            previous.index + 1,
            vec![],
            "BAD_HASH".to_string()
        )
    }


    pub fn chain_of(n: usize) -> Vec<Block> {
        let mut chain = vec![Self::genesis()];
        for _ in 1..n {
            let last = chain.last().unwrap();
            chain.push(Self::valid_after(last));
        }
        chain
    }
}