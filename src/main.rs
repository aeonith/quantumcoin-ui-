mod wallet;

use crate::wallet::Wallet;
use std::fs;

fn main() {
    println!("🚀 QuantumCoin Engine Initialized");

    // Load or create wallet
    let wallet_path = "wallet.json";

    let wallet = if fs::metadata(wallet_path).is_ok() {
        match Wallet::load_from_file(wallet_path) {
            Some(w) => {
                println!("🔐 Wallet loaded successfully.");
                w
            },
            None => {
                println!("❌ Failed to load wallet. Creating a new one...");
                let w = Wallet::new();
                if w.save_to_file(wallet_path) {
                    println!("💾 New wallet created and saved.");
                }
                w
            }
        }
    } else {
        println!("📁 No wallet file found. Creating new wallet...");
        let w = Wallet::new();
        if w.save_to_file(wallet_path) {
            println!("💾 New wallet saved successfully.");
        }
        w
    };

    // Show public address
    println!("🔑 Public Wallet Address: {}", wallet.public_key);

    // Simulate a message signing & verification test
    let message = b"QuantumCoin genesis";
    match wallet.sign_message(message) {
        Some(signature) => {
            let valid = wallet.verify_signature(message, &signature);
            println!("✅ Signature valid: {}", valid);
        },
        None => println!("❌ Failed to sign message."),
    }
}