mod blockchain;
mod cli;
mod secure_wallet;
mod mining;
mod validator;
mod revstop;
mod network;
mod peer;

use std::sync::{Arc, Mutex};
use secure_wallet::SecureWallet;
use blockchain::Blockchain;

fn main() {
    println!("🔐 QuantumCoin Node Starting...");

    let password = prompt_password();

    let wallet = SecureWallet::load(&password).unwrap_or_else(|| {
        println!("🔑 No wallet found. Creating new one...");
        let w = SecureWallet::generate(&password);
        w
    });

    let blockchain = Blockchain::load_from_disk().unwrap_or_else(|| {
        println!("🧱 No blockchain found. Creating genesis block...");
        Blockchain::new(&wallet)
    });

    let blockchain = Arc::new(Mutex::new(blockchain));
    let wallet = Arc::new(wallet);

    // Start CLI thread
    let cli_blockchain = Arc::clone(&blockchain);
    let cli_wallet = Arc::clone(&wallet);
    std::thread::spawn(move || {
        cli::start(cli_wallet, cli_blockchain);
    });

    // Start networking thread
    let net_blockchain = Arc::clone(&blockchain);
    let net_wallet = Arc::clone(&wallet);
    std::thread::spawn(move || {
        network::start_networking(net_wallet, net_blockchain);
    });

    loop {
        std::thread::park();
    }
}

fn prompt_password() -> String {
    use rpassword::read_password;
    println!("Please enter your wallet password:");
    read_password().unwrap_or_else(|_| {
        println!("Failed to read password.");
        std::process::exit(1);
    })
}