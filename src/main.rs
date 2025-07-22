mod wallet;
use wallet::Wallet;

use std::net::TcpListener;
use std::io::Write;

fn main() {
    println!("🚀 QuantumCoin Node Booting...");

    // Initialize or load wallet
    let wallet = Wallet::init_wallet();

    println!("💳 Wallet Address: {}", wallet.get_address());

    // Web listener setup (8080)
    let listener = TcpListener::bind("0.0.0.0:8080").expect("Failed to bind to port 8080");
    println!("🌐 QuantumCoin Web Server Running at http://0.0.0.0:8080");

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let response = b"QuantumCoin Web Node is live\n";
            stream.write_all(response).unwrap();
        }
    }
}