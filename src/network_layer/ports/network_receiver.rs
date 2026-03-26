pub trait NetworkReceiver {
    fn receive(&mut self) -> Option<(String, String)>;
}
