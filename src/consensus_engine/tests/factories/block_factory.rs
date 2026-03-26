use crate::consensus_engine::block::block::Block;
use crate::consensus_engine::transaction::transaction::Transaction;

pub struct BlockFactory;

impl BlockFactory {
    pub fn genesis() -> Block {
        Block::new(
            0,
            vec![],
            "".to_string()
        )
    }

    pub fn generate_a_correct_block(previous: &Block) -> Block {
        Block::new(
            previous.index + 1,
            vec![],
            previous.hash.to_string()
        )
    }

    pub fn generate_a_block_with_transactions(previous: &Block) -> Block {
        Block::new(
            previous.index + 1,
            Self::generate_random_transactions(),
            previous.hash.to_string()
        )
    }

    fn generate_random_transactions() -> Vec<Transaction> {
        vec![
            Transaction::new("0xAlice".into(), "0xBob".into(), 50, "sig1".into()),
            Transaction::new("0xBob".into(), "0xCarlos".into(), 25, "sig2".into()),
        ]
    }

    pub fn generate_a_block_with_mining_invalid_pow(previous: &Block) -> Block {
        let mut b = Self::generate_a_correct_block(previous);
        b.hash = "123456".to_string();
        b
    }


    pub fn generate_a_block_with_an_invalid_previous_hash(previous: &Block) -> Block {
        Block::new(
            previous.index + 1,
            vec![],
            "BAD_HASH".to_string()
        )
    }


    pub fn generate_a_chain_structurally_correct(n: usize) -> Vec<Block> {
        let mut chain = vec![Self::genesis()];
        for _ in 1..n {
            let last = chain.last().unwrap();
            chain.push(Self::generate_a_correct_block(last));
        }
        chain
    }
}