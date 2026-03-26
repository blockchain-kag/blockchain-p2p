#[derive(Debug, PartialEq)]
pub enum MempoolError {
    Full,
    Duplicate,
    InvalidTransaction(String),
}

impl std::fmt::Display for MempoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MempoolError::Full => write!(f, "Mempool llena"),
            MempoolError::Duplicate => write!(f, "Transacción duplicada"),
            MempoolError::InvalidTransaction(reason) => {
                write!(f, "Transacción inválida: {}", reason)
            }
        }
    }
}
