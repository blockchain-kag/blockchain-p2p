use std::{thread, time::Duration};

use blockchain_p2p::{
    network_layer::ports::{network_receiver::NetworkReceiver, network_sender::NetworkSender},
    network_layer_adapters::tcp::{tcp_receiver::TcpReceiver, tcp_sender::TCPSender},
};

fn main() {
    let sender = TCPSender::new();
    let mut receiver = TcpReceiver::new("127.0.0.1:9000");

    // Spawn test client
    thread::spawn(move || {
        let addr = String::from("127.0.0.1:9000");
        let msg = String::from("Hello, server!\n");
        sender.send(addr.clone(), msg).unwrap();
        thread::sleep(Duration::from_millis(5));
        let msg = String::from("Second message\n");
        sender.send(addr.clone(), msg).unwrap();
    });

    // Read messages from receiver
    for i in 0..2 {
        println!("{}", i);
        loop {
            if let Some((addr, msg)) = receiver.receive() {
                println!("Received from {}: {}", addr, msg);
                break;
            } else {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }

    println!("Test finished.");
}
