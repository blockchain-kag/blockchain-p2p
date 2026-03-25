use crate::consensus_engine::block::block::Block;

pub struct BlockValidator;

// todo: quiza podemos manejarlo con un Result<bool>
// Y ser mas informativo respecto a porque el no es valido el bloque

impl BlockValidator {
    // fixme, no necesariamente tenemos que tener un previous
    // Que pasa si es el primer bloque?
    // fixme
    pub fn validate(current: &Block, previous: &Block) -> bool {
        if current.hash != current.calculate_hash() { return false; }
        if current.previous_hash != previous.hash { return false; }
        if current.index != previous.index + 1 { return false; }
        true
    }
}