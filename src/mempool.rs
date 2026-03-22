//! # Mempool
//!
//! La mempool es la memoria temporal donde viven las transacciones pendientes
//! antes de ser incluidas en un bloque.
//!
//! ## Quién la usa
//! - **Networking Layer** → llama a `add_transaction()` cuando llega una tx de la red.
//! - **Consensus Engine** → llama a `get_transactions_for_block()` para armar un bloque,
//!   y luego a `remove_transactions()` una vez que el bloque fue minado y aceptado.
//!
//! ## Garantías
//! - No acepta transacciones duplicadas (misma tx_id).
//! - No acepta transacciones inválidas (campos vacíos, monto cero, mismo from/to).
//! - Tiene un tope máximo de transacciones (`MAX_MEMPOOL_SIZE`).

use std::collections::HashMap;

use crate::transaction::Transaction;

/// Cantidad máxima de transacciones que puede tener la mempool al mismo tiempo.
/// En Bitcoin son ~300.000. Para el TP usamos un valor chico y claro.
pub const MAX_MEMPOOL_SIZE: usize = 100;

/// Errores posibles al interactuar con la mempool.
#[derive(Debug, PartialEq)]
pub enum MempoolError {
    /// La mempool llegó a su capacidad máxima.
    Full,
    /// La transacción ya existe en la mempool (misma tx_id).
    Duplicate,
    /// La transacción no pasó la validación básica.
    InvalidTransaction(String),
}

impl std::fmt::Display for MempoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MempoolError::Full => write!(f, "Mempool llena, no se puede agregar la transacción"),
            MempoolError::Duplicate => write!(f, "La transacción ya está en la mempool"),
            MempoolError::InvalidTransaction(reason) => {
                write!(f, "Transacción inválida: {}", reason)
            }
        }
    }
}

/// La Mempool almacena transacciones pendientes indexadas por su tx_id.
/// Usar HashMap permite inserción y búsqueda en O(1).
pub struct Mempool {
    /// Clave: tx_id (hash SHA-256 del contenido de la tx).
    /// Valor: la transacción en sí.
    transactions: HashMap<String, Transaction>,
    /// Límite máximo de transacciones.
    max_size: usize,
}

impl Mempool {
    /// Crea una mempool vacía con el tamaño máximo por defecto.
    pub fn new() -> Self {
        Mempool {
            transactions: HashMap::new(),
            max_size: MAX_MEMPOOL_SIZE,
        }
    }

    /// Crea una mempool con tamaño máximo personalizado.
    /// Útil en tests para forzar el comportamiento de "mempool llena".
    pub fn with_max_size(max_size: usize) -> Self {
        Mempool {
            transactions: HashMap::new(),
            max_size,
        }
    }

    // -------------------------------------------------------------------------
    // API pública
    // -------------------------------------------------------------------------

    /// Agrega una transacción a la mempool.
    ///
    /// El orden de validación es importante:
    /// 1. Validación de campos → detecta errores del cliente rápido.
    /// 2. Control de capacidad → evita procesar si ya estamos llenos.
    /// 3. Detección de duplicados → evita trabajo redundante.
    /// 4. Inserción.
    ///
    /// Retorna el `tx_id` si tuvo éxito, o un `MempoolError` si falló.
    pub fn add_transaction(&mut self, tx: Transaction) -> Result<String, MempoolError> {
        // Paso 1: validar campos básicos
        Self::validate(&tx)?;

        // Paso 2: verificar capacidad
        if self.transactions.len() >= self.max_size {
            return Err(MempoolError::Full);
        }

        // Paso 3: detectar duplicados
        let tx_id = tx.tx_id();
        if self.transactions.contains_key(&tx_id) {
            return Err(MempoolError::Duplicate);
        }

        // Paso 4: insertar
        self.transactions.insert(tx_id.clone(), tx);
        Ok(tx_id)
    }

    /// Retorna hasta `limit` transacciones para que el consensus engine
    /// arme el próximo bloque.
    ///
    /// IMPORTANTE: NO elimina las transacciones de la mempool.
    /// La eliminación ocurre en `remove_transactions()`, recién cuando
    /// el bloque fue minado y aceptado por la red.
    pub fn get_transactions_for_block(&self, limit: usize) -> Vec<Transaction> {
        self.transactions.values().take(limit).cloned().collect()
    }

    /// Elimina de la mempool las transacciones que fueron incluidas en un bloque.
    /// Llamado por el consensus engine después de minar (o recibir) un bloque válido.
    ///
    /// Las transacciones que no estén en la mempool se ignoran silenciosamente
    /// (puede pasar si el bloque vino de otro nodo y nosotros no teníamos esas txs).
    pub fn remove_transactions(&mut self, transactions: &[Transaction]) {
        for tx in transactions {
            self.transactions.remove(&tx.tx_id());
        }
    }

    /// Retorna la cantidad actual de transacciones en la mempool.
    pub fn get_size(&self) -> usize {
        self.transactions.len()
    }

    /// Limpieza defensiva: elimina transacciones que ya no pasan la validación.
    ///
    /// En condiciones normales no debería haber transacciones inválidas aquí
    /// (porque `add_transaction` ya valida). Pero puede ser útil si las
    /// reglas de validación cambian en caliente, o para mantenimiento periódico.
    pub fn clear_invalid_transactions(&mut self) {
        self.transactions.retain(|_, tx| Self::validate(tx).is_ok());
    }

    // -------------------------------------------------------------------------
    // Validación interna (función asociada, no método, para poder usarla
    // dentro de `retain` sin problemas de borrow checker)
    // -------------------------------------------------------------------------

    fn validate(tx: &Transaction) -> Result<(), MempoolError> {
        if tx.from.is_empty() {
            return Err(MempoolError::InvalidTransaction(
                "'from' no puede estar vacío".into(),
            ));
        }
        if tx.to.is_empty() {
            return Err(MempoolError::InvalidTransaction(
                "'to' no puede estar vacío".into(),
            ));
        }
        if tx.from == tx.to {
            return Err(MempoolError::InvalidTransaction(
                "'from' y 'to' no pueden ser iguales".into(),
            ));
        }
        if tx.amount == 0 {
            return Err(MempoolError::InvalidTransaction(
                "el monto debe ser mayor a 0".into(),
            ));
        }
        if tx.sig.is_empty() {
            return Err(MempoolError::InvalidTransaction(
                "la firma no puede estar vacía".into(),
            ));
        }
        Ok(())
    }
}

impl Default for Mempool {
    fn default() -> Self {
        Self::new()
    }
}
