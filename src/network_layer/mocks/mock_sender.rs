use crate::network_layer::traits::network_sender::NetworkSender;

pub struct MockNetworkSender {
    is_result_correct: bool,
}

impl MockNetworkSender {
    pub fn new(is_result_correct: bool) -> MockNetworkSender {
        MockNetworkSender { is_result_correct }
    }
}

impl NetworkSender for MockNetworkSender {
    fn send(&self, _: String, _: String) -> Result<(), String> {
        if self.is_result_correct {
            Ok(())
        } else {
            Err(String::from("Error sending msg"))
        }
    }
}
