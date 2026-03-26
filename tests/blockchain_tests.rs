use blockchain_p2p::block::Block;
use blockchain_p2p::transaction::Transaction;

#[test]
fn block_hash_should_not_be_empty() {
    let tx = Transaction::new(
        "0xRocio".into(),
        "0xPedro".into(),
        100,
        "0xfirma".into(),
    );

    let block = Block::new(1, vec![tx], "prev".into());

    assert!(!block.hash.is_empty());
}
