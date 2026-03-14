use blockchain_p2p::block::Block;

#[test]
fn block_hash_should_not_be_empty() {

    let block = Block::new(
        1,
        vec!["tx".to_string()],
        "prev".to_string(),
    );

    assert!(!block.hash.is_empty());
}