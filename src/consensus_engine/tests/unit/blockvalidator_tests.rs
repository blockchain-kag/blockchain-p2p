use crate::consensus_engine::block::block::Block;
use crate::consensus_engine::tests::factories::block_factory::BlockFactory;
use crate::consensus_engine::validation::block_validator::BlockValidator;


#[test]
fn validator_should_accept_genesis_block() {
    let genesis = BlockFactory::genesis();
    assert!(BlockValidator::validate(&genesis, None));
}

#[test]
fn validator_should_reject_non_genesis_when_previous_is_none() {
    let block = Block::new(5, vec![], "".to_string());
    assert!(!BlockValidator::validate(&block, None));
}


#[test]
fn reject_block_with_wrong_previous_hash(){
    let b1 = BlockFactory::genesis();
    let b2 = BlockFactory::generate_a_block_with_an_invalid_previous_hash(&b1);
    assert!(BlockValidator::validate(&b2, Some(&b1)));
}

#[test]
fn reject_block_with_invalid_pow(){
    let b1 = BlockFactory::genesis();
    let b2 = BlockFactory::generate_a_block_with_mining_invalid_pow(&b1);
    assert!(BlockValidator::validate(&b2, Some(&b1)));
}

