mod wallet;
mod revstop;
mod cli;
mod blockchain;

use wallet::Wallet;
use revstop::RevStop;
use cli::run_cli;

fn main() {
    println!("🚀 QuantumCoin CLI Web Server Launched");

    // Load wallet and RevStop state
    let mut wallet = Wallet::load_from_files().expect("Failed to load wallet");
    let mut revstop = RevStop::load_status().unwrap_or_else(|_| {
        println!("⚠️  RevStop status not found, defaulting to unlocked.");
        RevStop::new()
    });

    // Run CLI interface
    run_cli(&mut wallet, &mut revstop);
}