mod block;
mod blockchain;
mod cli;
mod transaction;
mod wallet;
mod revstop;
mod peer;
mod network;

use blockchain::Blockchain;
use wallet::Wallet;
use std::sync::{Arc, Mutex};

fn main() {
    println!("🔐 QuantumCoin Node Starting...");

    let wallet = Wallet::load_from_files().unwrap_or_else(|_| {
        println!("🔑 No wallet found. Creating new one...");
        let w = Wallet::new();
        w.save_to_files().unwrap();
        w
    });

    let blockchain = Blockchain::load_from_disk().unwrap_or_else(|| {
        println!("🧱 No blockchain found. Creating genesis block...");
        Blockchain::new(&wallet)
    });

    let blockchain = Arc::new(Mutex::new(blockchain));
    let wallet = Arc::new(wallet.clone());

    // 🧠 Start CLI thread
    let cli_blockchain = Arc::clone(&blockchain);
    let cli_wallet = Arc::clone(&wallet);
    std::thread::spawn(move || {
        cli::start(cli_wallet, cli_blockchain);
    });

    // 🌐 Start network thread
    let net_blockchain = Arc::clone(&blockchain);
    let net_wallet = Arc::clone(&wallet);
    std::thread::spawn(move || {
        network::start_networking(net_wallet, net_blockchain);
    });

    // Keep alive
    loop {
        std::thread::park();
    }
}