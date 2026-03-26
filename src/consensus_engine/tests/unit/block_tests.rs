use crate::consensus_engine::block::block::Block;
use crate::consensus_engine::tests::factories::block_factory::BlockFactory;
use crate::consensus_engine::traits::hasher::{Hasher, Sha256Hasher};


#[test]
fn block_should_have_a_correct_index(){
    let previous_block = BlockFactory::genesis();
    let block = BlockFactory::generate_a_correct_block(&previous_block);
    assert!(block.index == previous_block.index + 1);
}


#[test]
fn block_should_have_a_correct_previous_hash(){
    let block = Block::new(5, vec![], "abc".to_string());
    assert_eq!(block.previous_hash, "abc".to_string());
}


#[test]
fn new_block_check_timestamp(){
    use std::time::{SystemTime, UNIX_EPOCH};
    let before = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let block = Block::new(0, vec![], "".to_string());
    let after = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

    assert!(block.timestamp >= before && block.timestamp <= after);
}


#[test]
fn block_should_have_a_correct_hash() {
    let previous = BlockFactory::genesis();
    let block = BlockFactory::generate_a_correct_block(&previous);
    let recalculated = Sha256Hasher::hash_block(&block);
    assert_eq!(block.hash, recalculated);
}

#[test]
fn block_should_have_a_zero_nonce_value() { // not mining yet => nonce = 0
    let previous = BlockFactory::genesis();
    let block = BlockFactory::generate_a_correct_block(&previous);

    assert_eq!(block.nonce, 0);
}