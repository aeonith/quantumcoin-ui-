mod wallet;
mod block;
mod transaction;
mod blockchain;

use wallet::Wallet;
use blockchain::Blockchain;

fn main() {
    println!("🚀 QuantumCoin Engine Booted");

    let wallet = Wallet::new();
    println!("🧠 Wallet initialized: {}", wallet.get_address());

    let mut blockchain = Blockchain::new();
    blockchain.print_chain();
}