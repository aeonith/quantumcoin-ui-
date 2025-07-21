use std::net::TcpListener;
use std::io::{Read, Write};

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").expect("Failed to bind to port 8080");
    println!("🚀 QuantumCoin Web Server Running at http://0.0.0.0:8080");

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let mut buffer = [0; 512];
            let _ = stream.read(&mut buffer);
            let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nQuantumCoin Server is Live!";
            let _ = stream.write(response.as_bytes());
            let _ = stream.flush();
        }
    }
}