pub trait NetworkSender {
    fn send(&self, identifier: String, msg: String) -> Result<(), String>;
}

pub trait NetworkReceiver {
    fn receive(&self) -> Option<(String, String)>;
}
