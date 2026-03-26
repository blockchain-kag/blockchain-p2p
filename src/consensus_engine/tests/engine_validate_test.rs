use crate::consensus_engine::block::block::Block;

// todo: make a factory that create random blocks
// and check all this cases!


// fixme
#[test]
fn validate_index() {
    let last_block = Block::new(
        1, 
        vec![],
        "0x00".to_string()
    );

    let new_block = Block::new(
        2, 
        vec![], 
        last_block.hash.to_string()
    );

    assert_eq!(new_block.index, last_block.index + 1);
}

#[test]
fn validate_timestamp(){
    // let last_block = Block::new(
    //
    // );
    //
    // let  new_block = Block::new(
    //
    // );
    //
    // assert!(new_block.timestamp > last_block.timestamp);
}

#[test]
fn validate_hash(){
    // let recalculated_hash = calculate_hash();
    //
    // // Para el mismo previous_hash, deberia darme el mismo hash
    //
    // assert_eq!(recalculated_hash, block.hash);
}


/*
  (validar que el campo previousHash tenga el formato correcto)

  Este test NO verifica que coincida con el último bloque.
  Solo verifica que el valor del campo previous_hash es válido internamente.

  Ejemplos de validaciones:

  Que no sea ""
  Que sea un string de longitud esperada (64 caracteres si usás SHA-256)
  Que solo tenga caracteres hexadecimales
  Que no sea None

  No compara con otro bloque.
  Esto es “validación de estructura”.

  */
// “¿El campo previous_hash tiene un formato válido?”
#[test]
fn validate_previous_hash(){

}

#[test]
fn validate_if_hash_correctly_calculated(){

}

#[test]
fn validate_if_previous_hash_match_with_last_block(){

}


// Proof of Work Validation
// which cases can we have?

// Para evitar que la red se llene de bloques basura (spam), implementaremos una dificultad
// básica. El hash del bloque debe empezar con una cantidad de ceros definida (ej. "000").

// todo: struct better this tests.

#[test]
fn should_pass_ok_when_you_have_a_valid_hash_in_block(){
    // block.hash.starts_with(INITIAL_DIGITS)
}

#[test]
fn should_throws_error_when_you_have_a_invalid_hash_in_block(){
    // block.hash.starts_with(INITIAL_DIGITS)
}


// Transaction Validation
// which cases can we have?
fn validate_transaction(){

}

