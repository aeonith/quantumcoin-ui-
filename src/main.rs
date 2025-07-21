mod wallet;
mod transaction;

use wallet::Wallet;
use transaction::Transaction;
use std::fs;
use std::path::Path;

fn main() {
    println!("🚀 QuantumCoin Engine Initialized");

    let pub_path = "wallet_public.key";
    let priv_path = "wallet_private.key";

    let wallet = if Path::new(pub_path).exists() && Path::new(priv_path).exists() {
        println!("🔑 Loading wallet from files...");
        Wallet::load_from_files(pub_path, priv_path).expect("❌ Failed to load wallet files")
    } else {
        println!("🆕 Generating new wallet...");
        let wallet = Wallet::new();
        wallet.save_to_files(pub_path, priv_path).expect("❌ Failed to save wallet files");
        wallet
    };

    println!("✅ Wallet initialized");

    let sender_address = base64::encode(wallet.public_key.as_bytes());
    let recipient_address = "SampleRecipientAddress123".to_string();

    let mut tx = Transaction::new(sender_address.clone(), recipient_address.clone(), 100.0);
    let signature = wallet.sign_transaction(&tx);
    tx.set_signature(signature.clone());

    println!("\n🧾 Transaction created:");
    println!("From: {}", tx.sender);
    println!("To: {}", tx.recipient);
    println!("Amount: {}", tx.amount);
    println!("Signature (base64): {:?}", tx.signature.as_ref().unwrap());

    let tx_bytes = serde_json::to_vec(&tx).expect("Serialization failed");

    let verified = wallet.verify_signature(&tx_bytes, &signature);
    println!("\n🔐 Signature verified: {}", verified);
}