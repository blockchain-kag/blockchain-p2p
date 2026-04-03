use std::io::{Write, stdout};

use crate::node::ports::emmitter::Emmitter;

#[derive(Default)]
pub struct ConsoleEmmitter {}

impl Emmitter for ConsoleEmmitter {
    fn emmit(&mut self, msg: String) -> Result<(), String> {
        print!("{}", msg);
        {
            let this = stdout().flush();
            match this {
                Ok(_) => Ok(()),
                Err(e) => Err(e.to_string()),
            }
        }
    }
}
