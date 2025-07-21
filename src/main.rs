mod wallet;
mod blockchain;
mod transaction;
mod revstop;

use wallet::*;
use blockchain::*;
use transaction::*;
use revstop::*;

fn main() {
    println!("🚀 QuantumCoin Engine Initialized");
    
    // Load wallet
    let wallet = load_from_files("wallet_public.key", "wallet_private.key");
    println!("🔐 Wallet loaded: {}", wallet.get_address());

    // Load blockchain
    let mut blockchain = load_blockchain_from_file("blockchain.json").unwrap_or_else(|_| {
        println!("🧱 No blockchain found. Creating new one...");
        Blockchain::new()
    });

    // Show RevStop status
    let revstop_status = load_status("revstop_status.json").unwrap_or(false);
    println!("🛡️ RevStop Enabled: {}", revstop_status);

    // Display balance
    let balance = wallet.get_balance(&blockchain);
    println!("💰 Balance: {} QTC", balance);

    // Save blockchain state
    save_blockchain_to_file(&blockchain, "blockchain.json");
}