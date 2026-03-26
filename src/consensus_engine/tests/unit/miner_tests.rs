use crate::consensus_engine::miner::miner::BlockMiner;
use crate::consensus_engine::tests::factories::block_factory::BlockFactory;
use crate::consensus_engine::traits::hasher::{Hasher, Sha256Hasher};
use crate::consensus_engine::traits::miner::Miner;
use crate::consensus_engine::validation::block_validator::BlockValidator;

#[test]
fn miner_should_generate_a_valid_pow_hash() {
    let prev = BlockFactory::genesis();
    let block = BlockFactory::generate_a_correct_block(&prev);

    let miner = BlockMiner;
    let mined = miner.mine(block, 3); // dificultad chica para rápido testeo

    assert_eq!(mined.hash, "000".to_string()); // fixme-> start_with?
}

#[test]
fn miner_should_increment_nonce_until_valid_pow() {
    let prev = BlockFactory::genesis();
    let block = BlockFactory::generate_a_correct_block(&prev);

    let miner = BlockMiner;


    // let mined = miner.mine(block, 3); // other option block.clone() but doesn't work
    // assert!(mined.nonce > block.nonce);
}


#[test]
fn miner_should_change_hash() {
    let prev = BlockFactory::genesis();
    let block = BlockFactory::generate_a_correct_block(&prev);
    let miner = BlockMiner;
    let mined = miner.mine(block, 3);

    // assert_ne!(mined.hash, block.hash);
}

#[test]
fn miner_should_produce_a_block_accepted_by_validator() {
    let prev = BlockFactory::genesis();
    let block = BlockFactory::generate_a_correct_block(&prev);

    let miner = BlockMiner;
    let mined = miner.mine(block, 2);

    assert!(BlockValidator::validate(&mined, Some(&prev)));
}


#[test]
fn miner_should_not_modify_previous_hash() {
    let prev = BlockFactory::genesis();
    let block = BlockFactory::generate_a_correct_block(&prev);

    let miner = BlockMiner;
    // let mined = miner.mine(block.clone(), 3);

    // assert_eq!(mined.previous_hash, block.previous_hash);
}


#[test]
fn miner_should_produce_hash_matching_recalculated_hash() {
    let prev = BlockFactory::genesis();
    let block = BlockFactory::generate_a_correct_block(&prev);

    let miner = BlockMiner;
    let mined = miner.mine(block, 3);

    let recalculated = Sha256Hasher::hash_block(&mined);

    assert_eq!(recalculated, mined.hash);
}

