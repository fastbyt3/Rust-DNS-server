use std::net::UdpSocket;

use dns_starter_rust::Header;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);
                let header = Header {
                    id: 1234,
                    qr: dns_starter_rust::QueryResponseIndicator::Response,
                    opcode: dns_starter_rust::OpCode::Status,
                    aa: false,
                    tc: false,
                    rd: false,
                    ra: false,
                    z: false,
                    rcode: dns_starter_rust::ResponseCode::NoError,
                    qdcount: 0,
                    ancount: 0,
                    nscount: 0,
                    arcount: 0,
                };
                udp_socket
                    .send_to(&header.to_bytes(), source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
