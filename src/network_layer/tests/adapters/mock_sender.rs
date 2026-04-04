use crate::network_layer::ports::network_sender::NetworkSender;

pub struct MockNetworkSender {
    is_result_correct: bool,
}

impl MockNetworkSender {
    pub fn new(is_result_correct: bool) -> MockNetworkSender {
        MockNetworkSender { is_result_correct }
    }
}

impl NetworkSender for MockNetworkSender {
    fn send(&self, _: String, _: String) -> Result<String, String> {
        if self.is_result_correct {
            Ok(String::from("Correct send"))
        } else {
            Err(String::from("Err"))
        }
    }
}
