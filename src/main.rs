mod wallet;
mod blockchain;
mod transaction;
mod revstop;
mod cli;

use wallet::Wallet;
use blockchain::Blockchain;
use transaction::Transaction;
use revstop::RevStop;
use cli::start_cli;

use std::sync::{Arc, Mutex};
use std::fs;

fn main() {
    println!("🚀 QuantumCoin Engine Initialized");

    // Load or create wallet
    let wallet = match Wallet::load_from_files() {
        Ok(w) => w,
        Err(_) => {
            println!("🔐 No wallet found. Generating new wallet...");
            let new_wallet = Wallet::generate();
            new_wallet.save_to_files().expect("Failed to save wallet");
            println!("✅ New wallet generated and saved.");
            new_wallet
        }
    };

    // Load or create blockchain
    let blockchain = match Blockchain::load_from_disk() {
        Ok(bc) => bc,
        Err(_) => {
            println!("📦 No blockchain found. Creating genesis block...");
            let mut bc = Blockchain::new();

            let genesis_tx = Transaction {
                sender: "Genesis".to_string(),
                recipient: wallet.get_address(),
                amount: 1_250_000.0,
                signature: None,
            };

            bc.add_genesis_block(genesis_tx);
            bc.save_to_disk().expect("Failed to save blockchain");
            println!("✅ Genesis block created and blockchain initialized.");
            bc
        }
    };

    // Load or create RevStop protection
    let revstop = match RevStop::load_status() {
        Ok(rs) => rs,
        Err(_) => {
            println!("🔐 RevStop not initialized. Starting in unlocked mode.");
            let rs = RevStop::default();
            rs.save_status().expect("Failed to save RevStop status");
            rs
        }
    };

    // Shared access to components
    let wallet = Arc::new(Mutex::new(wallet));
    let blockchain = Arc::new(Mutex::new(blockchain));
    let revstop = Arc::new(Mutex::new(revstop));

    // Launch CLI interface
    start_cli(wallet, blockchain, revstop);
}