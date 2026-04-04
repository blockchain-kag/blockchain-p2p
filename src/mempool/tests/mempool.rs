use crate::{
    common::types::tx::{Hash, Tx},
    mempool::types::mempool::Mempool,
};

#[test]
fn mempool() {
    let mut mempool = Mempool::default();
    assert!(mempool.get_first_n(1).is_empty());
    mempool.push(Tx::new(Hash::zero(), vec![], vec![], 0, vec![]));
    assert!(!mempool.get_first_n(1).is_empty());
    assert!(mempool.get_first_n(1).is_empty());
}
