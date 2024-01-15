use std::net::UdpSocket;

use dns_starter_rust::Message;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let resolver_flag = std::env::args().nth(1).unwrap() == "--resolver";

    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);
                let mut msg = Message::from_bytes(buf);
                // msg.prepare_response();
                if resolver_flag == true {
                    let forwarding_ip = std::env::args().nth(2).unwrap();
                    println!("Forwarding requests to {forwarding_ip}");
                    msg.forward_requests_to(forwarding_ip);
                } else {
                    msg.prepare_response();
                }
                udp_socket
                    .send_to(&msg.to_bytes(), source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
