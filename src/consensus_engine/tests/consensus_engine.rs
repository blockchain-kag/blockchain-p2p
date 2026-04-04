use crate::{
    common::types::{block::Block, tx::Hash},
    consensus_engine::{
        tests::adapters::{
            block_validator::ValidBlockValidator,
            miner::{CorretlyMinedBlockMiner, ZeroHasher},
        },
        types::consensus_engine::ConsensusEngine,
    },
};

#[test]
fn mines_and_validates_block() {
    let difficulty = 3;
    let engine = ConsensusEngine::new(
        Box::new(CorretlyMinedBlockMiner()),
        Box::new(ValidBlockValidator()),
        difficulty,
    );
    let hasher = ZeroHasher();
    let genesis_block = Block::new(0, Hash::zero(), 0, vec![], &hasher);
    let new_block = engine.mine(vec![], &genesis_block, &hasher);
    assert_eq!(genesis_block.hash(&hasher), new_block.header.prev_hash);
    assert!(engine.validate(&genesis_block, &new_block))
}
