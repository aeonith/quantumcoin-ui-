mod wallet;
mod revstop;
mod blockchain;
mod transaction;
mod cli;

use wallet::Wallet;
use revstop::RevStop;
use blockchain::Blockchain;

fn main() {
    println!("🚀 QuantumCoin Engine Initialized");

    // Load the user's wallet and RevStop
    let mut wallet = Wallet::load_from_files().unwrap_or_else(|_| {
        println!("🧾 Creating a new wallet...");
        let new_wallet = Wallet::generate();
        new_wallet.save_to_files().expect("❌ Failed to save wallet.");
        new_wallet
    });

    let mut revstop = RevStop::load_status().unwrap_or_else(|_| {
        println!("🔐 Initializing RevStop...");
        let mut rs = RevStop::new();
        rs.lock("default_password"); // This can be changed via CLI
        rs.save_status().expect("❌ Failed to save RevStop status.");
        rs
    });

    // Load or create blockchain
    let mut blockchain = Blockchain::load_from_file().unwrap_or_else(|_| {
        println!("🧱 No blockchain found. Creating Genesis block...");
        Blockchain::new_with_genesis(&wallet)
    });

    // Launch CLI
    cli::start_cli(&mut wallet, &mut blockchain, &mut revstop);
}