mod wallet;
use wallet::Wallet;
use std::sync::{Arc, Mutex};
use std::net::TcpListener;
use std::io::prelude::*;

fn main() {
    println!("🚀 QuantumCoin Web Server Running");

    // Initialize the wallet
    let wallet = Arc::new(Mutex::new(Wallet::load_from_file("wallet_key.json")
        .unwrap_or_else(|| Wallet::new())));

    // Show wallet address and balance
    let wallet_ref = wallet.lock().unwrap();
    println!("🔐 Wallet Address: {}", wallet_ref.get_address());
    println!("💰 Wallet Balance: {} QTC", wallet_ref.get_balance());
    drop(wallet_ref);

    // Start web server
    let listener = TcpListener::bind("0.0.0.0:8080").expect("❌ Failed to bind to port 8080");
    println!("==> Your service is live 🎉");
    println!("==> Available at your primary URL");

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            println!("✅ Incoming connection");
            let response = b"QuantumCoin Node Live\n";
            stream.write_all(response).unwrap();
        }
    }
}