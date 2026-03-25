use crate::consensus_engine::tests::factories::block_factory::BlockFactory;
use crate::consensus_engine::validation::block_validator::BlockValidator;

#[test]
fn validate_genesis_block() {
    let genesis = BlockFactory::genesis();
    assert!(BlockValidator::validate(&genesis, None));
}

#[test]
fn reject_block_with_wrong_previous_hash(){
    let b1 = BlockFactory::genesis();
    let b2 = BlockFactory::invalid_previous_hash(&b1);

    assert!(BlockValidator::validate(&b2, Some(&b1)));
}

#[test]
fn reject_block_with_invalid_pow(){
    let b1 = BlockFactory::genesis();
    let b2 = BlockFactory::invalid_pow(&b1);

    assert!(BlockValidator::validate(&b2, Some(&b1)));
}

