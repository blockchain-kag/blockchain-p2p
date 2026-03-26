use std::{io::Write, net::TcpStream, thread, time::Duration};

use blockchain_p2p::{
    network_layer::ports::network_receiver::NetworkReceiver,
    network_layer_adapters::tcp::tcp_receiver::TcpReceiver,
};

fn main() {
    let mut receiver = TcpReceiver::new("127.0.0.1:9000");

    // Spawn test client
    thread::spawn(|| {
        let mut stream = TcpStream::connect("127.0.0.1:9000").unwrap();
        stream.write_all(b"Hello, server!\n").unwrap();
        thread::sleep(Duration::from_millis(500)); // small delay
        stream.write_all(b"Second message\n").unwrap();
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
