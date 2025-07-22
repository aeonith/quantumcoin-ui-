use std::sync::{Arc, Mutex};

mod blockchain;
mod cli;
mod peer;
mod revstop;
mod transaction;
mod wallet;

use blockchain::Blockchain;
use wallet::Wallet;

fn main() {
    // === Load Wallet (from file or generate new) ===
    let wallet = Wallet::load_or_generate().expect("🛑 Failed to load or generate wallet.");

    // === Load Blockchain ===
    let blockchain = Blockchain::load_or_new();
    let blockchain = Arc::new(Mutex::new(blockchain));

    // === Start Peer-to-Peer Server in Background Thread ===
    {
        let chain_clone = Arc::clone(&blockchain);
        std::thread::spawn(move || {
            peer::start_peer_server(chain_clone, 6001);
        });
    }

    // === Launch CLI Interface ===
    cli::start_cli(wallet, blockchain);
}