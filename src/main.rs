mod wallet;
mod blockchain;

use wallet::Wallet;
use blockchain::Blockchain;
use std::fs;
use std::path::Path;
use std::time::Instant;

fn main() {
    println!("🚀 QuantumCoin Node Initializing...");

    // Load or create wallet
    let wallet_path_pub = "wallet_public.key";
    let wallet_path_priv = "wallet_private.key";

    let wallet = if Path::new(wallet_path_pub).exists() && Path::new(wallet_path_priv).exists() {
        Wallet::load_from_files(wallet_path_pub, wallet_path_priv)
            .expect("⚠️ Failed to load wallet from files.")
    } else {
        println!("🧾 No wallet found — generating new one...");
        let wallet = Wallet::new();
        wallet.save_to_files(wallet_path_pub, wallet_path_priv);
        println!("✅ Wallet saved to disk.");
        wallet
    };

    // Print wallet address only (no private key)
    let address = wallet.get_address();
    println!("🔐 Wallet Address: {}", address);

    // Load or create blockchain
    let start_time = Instant::now();
    let mut blockchain = Blockchain::load_from_disk().unwrap_or_else(|| {
        println!("📦 No blockchain found — creating genesis block...");
        Blockchain::new(address.clone()) // assign genesis to this wallet
    });

    println!("✅ Blockchain ready. Load time: {:.2?}", start_time.elapsed());

    // Example mining display (can be expanded)
    blockchain.mine_pending_transactions(address.clone());
    blockchain.save_to_disk();

    println!("✅ Block mined and saved. Chain is live.");
}