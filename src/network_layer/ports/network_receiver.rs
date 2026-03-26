pub trait NetworkReceiver: Send {
    fn receive(&mut self) -> Option<(String, String)>;
}
