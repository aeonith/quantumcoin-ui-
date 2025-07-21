use quantumcoin::{
    blockchain::Blockchain,
    mining::Mining,
    revstop::RevStop,
    transaction::Transaction,
    wallet::Wallet,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Load wallet and handle Result
    let wallet = Wallet::load_from_files("wallet.json")
        .expect("Failed to load wallet");
    println!("👛 Wallet loaded: {}", wallet.get_address());

    // Load blockchain
    let mut blockchain = Blockchain::load_from_file("blockchain.json");
    if blockchain.chain.is_empty() {
        blockchain.create_genesis_block(&wallet.get_address());
        println!("🌱 Genesis block created!");
    }

    // Initialize RevStop
    let mut revstop = RevStop::new();
    if revstop.is_locked() {
        println!("🔒 RevStop protection is enabled.");
    } else {
        println!("🔓 RevStop is currently disabled.");
    }

    // Sample transaction (for testing)
    let tx = wallet.create_transaction(
        "recipient_public_key_string",
        10.0,
    );
    blockchain.add_transaction(tx);

    // Mining simulation
    println!("⛏️  Starting mining...");
    let success = blockchain.mine_pending_transactions(&wallet.get_address());
    if success {
        println!("✅ Mining completed and block added.");
        blockchain.save_to_file("blockchain.json");
    } else {
        println!("⚠️  Mining failed or nothing to mine.");
    }

    Ok(())
}