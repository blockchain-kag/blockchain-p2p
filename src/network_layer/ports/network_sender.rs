pub trait NetworkSender: Send {
    fn send(&self, identifier: String, msg: String) -> Result<String, String>;
}
