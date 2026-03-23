pub trait NetworkSender {
    fn send(&self, identifier: String, msg: String) -> Result<(), String>;
}
