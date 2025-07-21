mod wallet;
mod transaction;
mod block;
mod blockchain;
mod revstop;
mod cli;

use wallet::Wallet;
use blockchain::Blockchain;
use std::fs;
use std::io::{self, Write};
use revstop::RevStop;
use cli::start_cli;

fn main() {
    println!("🚀 QuantumCoin Engine Initialized");

    let mut wallet = match Wallet::load_from_files() {
        Ok(w) => {
            println!("🔑 Wallet loaded.");
            w
        }
        Err(_) => {
            println!("⚠️  No wallet found. Generating new wallet...");
            let w = Wallet::generate();
            match w.save_to_files() {
                Ok(_) => println!("✅ New wallet saved."),
                Err(e) => println!("❌ Failed to save wallet: {}", e),
            }
            w
        }
    };

    println!("📬 Your wallet address: {}", wallet.get_address());

    // Step 1: Terms & Conditions agreement
    if !wallet.agreed_to_terms {
        println!("\n📄 Please agree to the QuantumCoin Terms & Conditions.");
        println!("Type 'yes' to accept:");
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if input.trim().to_lowercase() == "yes" {
            wallet.agree_to_terms();
        } else {
            println!("❌ You must accept terms to continue.");
            return;
        }
    }

    // Step 2: KYC verification
    if !wallet.kyc_verified {
        println!("\n🧾 Enter your KYC Verification Code (hint: try 'KYC123456'):");
        let mut code = String::new();
        io::stdin().read_line(&mut code).unwrap();
        if !wallet.verify_kyc(code.trim()) {
            println!("❌ Verification failed. Exiting.");
            return;
        }
    }

    // Step 3: Initialize RevStop
    let mut revstop = RevStop::load_status().unwrap_or_default();

    // Step 4: Load or initialize blockchain
    let mut blockchain = match Blockchain::load_from_file("blockchain.json") {
        Ok(chain) => {
            println!("⛓️  Blockchain loaded.");
            chain
        }
        Err(_) => {
            println!("⛓️  No blockchain found. Creating genesis block...");
            let mut new_chain = Blockchain::new();
            let genesis_tx = transaction::Transaction {
                sender: "GENESIS".to_string(),
                recipient: wallet.get_address(),
                amount: 1_250_000.0,
                signature: None,
            };
            new_chain.add_transaction(genesis_tx);
            new_chain.mine_pending_transactions("GENESIS".to_string());
            match new_chain.save_to_file("blockchain.json") {
                Ok(_) => println!("✅ Blockchain saved."),
                Err(e) => println!("❌ Blockchain save error: {}", e),
            }
            new_chain
        }
    };

    // Step 5: Start CLI
    start_cli(&mut wallet, &mut blockchain, &mut revstop);
}