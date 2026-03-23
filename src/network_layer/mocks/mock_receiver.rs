use crate::network_layer::traits::network_receiver::NetworkReceiver;

pub struct MockNetworkReceiver {
    has_received: bool,
}

impl MockNetworkReceiver {
    pub fn new(has_received: bool) -> MockNetworkReceiver {
        Self { has_received }
    }
}

impl NetworkReceiver for MockNetworkReceiver {
    fn receive(&self) -> Option<(String, String)> {
        if self.has_received {
            Some((String::from("sender"), String::from("msg")))
        } else {
            None
        }
    }
}
