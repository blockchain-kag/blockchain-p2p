use blockchain_p2p::mempool::{Mempool, MempoolError};
use blockchain_p2p::transaction::Transaction;

fn tx(from: &str, to: &str, amount: u64, sig: &str) -> Transaction {
    Transaction::new(from.into(), to.into(), amount, sig.into())
}

// -- add_transaction --

#[test]
fn agrega_transaccion_valida() {
    let pool = Mempool::new();
    let result = pool.add_transaction(tx("0xA", "0xB", 50, "sig1"));
    assert!(result.is_ok());
    assert_eq!(pool.get_size(), 1);
}

#[test]
fn rechaza_from_vacio() {
    let pool = Mempool::new();
    let err = pool.add_transaction(tx("", "0xB", 50, "sig1")).unwrap_err();
    assert!(matches!(err, MempoolError::InvalidTransaction(_)));
}

#[test]
fn rechaza_to_vacio() {
    let pool = Mempool::new();
    let err = pool.add_transaction(tx("0xA", "", 50, "sig1")).unwrap_err();
    assert!(matches!(err, MempoolError::InvalidTransaction(_)));
}

#[test]
fn rechaza_from_igual_a_to() {
    let pool = Mempool::new();
    let err = pool.add_transaction(tx("0xA", "0xA", 50, "sig1")).unwrap_err();
    assert!(matches!(err, MempoolError::InvalidTransaction(_)));
}

#[test]
fn rechaza_monto_cero() {
    let pool = Mempool::new();
    let err = pool.add_transaction(tx("0xA", "0xB", 0, "sig1")).unwrap_err();
    assert!(matches!(err, MempoolError::InvalidTransaction(_)));
}

#[test]
fn rechaza_firma_vacia() {
    let pool = Mempool::new();
    let err = pool.add_transaction(tx("0xA", "0xB", 50, "")).unwrap_err();
    assert!(matches!(err, MempoolError::InvalidTransaction(_)));
}

#[test]
fn rechaza_transaccion_duplicada() {
    let pool = Mempool::new();
    pool.add_transaction(tx("0xA", "0xB", 50, "sig1")).unwrap();

    let err = pool.add_transaction(tx("0xA", "0xB", 50, "sig1")).unwrap_err();
    assert_eq!(err, MempoolError::Duplicate);
    assert_eq!(pool.get_size(), 1);
}

#[test]
fn rechaza_cuando_mempool_esta_llena() {
    let pool = Mempool::with_max_size(2);
    pool.add_transaction(tx("0xA", "0xB", 10, "s1")).unwrap();
    pool.add_transaction(tx("0xA", "0xB", 20, "s2")).unwrap();

    let err = pool.add_transaction(tx("0xA", "0xB", 30, "s3")).unwrap_err();
    assert_eq!(err, MempoolError::Full);
}

// -- get_transactions_for_block --

#[test]
fn get_transactions_respeta_el_limite() {
    let pool = Mempool::new();
    for i in 1..=5u64 {
        pool.add_transaction(tx("0xA", "0xB", i, &format!("sig{}", i))).unwrap();
    }

    let selected = pool.get_transactions_for_block(3);
    assert_eq!(selected.len(), 3);
}

#[test]
fn get_transactions_no_elimina_de_la_mempool() {
    let pool = Mempool::new();
    pool.add_transaction(tx("0xA", "0xB", 50, "sig1")).unwrap();

    let _ = pool.get_transactions_for_block(10);

    assert_eq!(pool.get_size(), 1);
}

// -- remove_transactions --

#[test]
fn remove_transactions_elimina_las_incluidas_en_el_bloque() {
    let pool = Mempool::new();
    let t1 = tx("0xA", "0xB", 50, "sig1");
    let t2 = tx("0xA", "0xC", 30, "sig2");

    pool.add_transaction(t1.clone()).unwrap();
    pool.add_transaction(t2.clone()).unwrap();
    assert_eq!(pool.get_size(), 2);

    pool.remove_transactions(&vec![t1]);

    assert_eq!(pool.get_size(), 1);
}

#[test]
fn remove_transactions_ignora_las_que_no_estaban() {
    let pool = Mempool::new();
    pool.add_transaction(tx("0xA", "0xB", 50, "sig1")).unwrap();

    pool.remove_transactions(&vec![tx("0xX", "0xY", 99, "sigX")]);

    assert_eq!(pool.get_size(), 1);
}

// -- flujo completo --

#[test]
fn flujo_completo_networking_mempool_consensus() {
    let pool = Mempool::new();

    let t1 = tx("0xRocio", "0xPedro", 100, "sig_r1");
    let t2 = tx("0xPedro", "0xJuan", 50, "sig_p1");
    let t3 = tx("0xJuan", "0xRocio", 25, "sig_j1");

    pool.add_transaction(t1.clone()).unwrap();
    pool.add_transaction(t2.clone()).unwrap();
    pool.add_transaction(t3.clone()).unwrap();
    assert_eq!(pool.get_size(), 3);

    let para_bloque = pool.get_transactions_for_block(2);
    assert_eq!(para_bloque.len(), 2);

    pool.remove_transactions(&para_bloque);

    assert_eq!(pool.get_size(), 1);
}
