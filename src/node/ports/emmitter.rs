pub trait Emmitter: Send + Sync {
    fn emmit(&mut self, msg: String) -> Result<(), String>;
}
