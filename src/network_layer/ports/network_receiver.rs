pub trait NetworkReceiver {
    fn receive(&self) -> Option<(String, String)>;
}
