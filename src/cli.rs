use crate::secure_wallet::SecureWallet;
use crate::blockchain::Blockchain;
use crate::mining;
use crate::validator;
use crate::revstop::RevStop;
use std::sync::{Arc, Mutex};
use std::io::{self, Write};

pub fn start(wallet: Arc<SecureWallet>, blockchain: Arc<Mutex<Blockchain>>) {
    let revstop = RevStop::load();

    loop {
        println!("\n=== QuantumCoin CLI ===");
        println!("1) Show Balance");
        println!("2) Send Coins");
        println!("3) Mine Pending Transactions");
        println!("4) Show Last 5 Transactions");
        println!("5) Export Wallet (2FA)");
        println!("6) Enable RevStop Lock");
        println!("7) Disable RevStop Lock");
        println!("8) Show Wallet Address");
        println!("9) Exit");
        print!("Enter choice: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        match choice.trim() {
            "1" => {
                let blockchain = blockchain.lock().unwrap();
                let balance = blockchain.get_balance(&wallet.get_address());
                println!("💰 Your balance: {:.8} QTC", balance as f64 / 100.0);
            }
            "2" => {
                println!("Enter recipient address:");
                let mut recipient = String::new();
                io::stdin().read_line(&mut recipient).unwrap();
                let recipient = recipient.trim();

                println!("Enter amount (QTC):");
                let mut amount = String::new();
                io::stdin().read_line(&mut amount).unwrap();
                let amount: u64 = (amount.trim().parse::<f64>().unwrap_or(0.0) * 100.0) as u64;

                let tx = wallet.create_transaction(recipient, amount);
                if validator::validate_transaction(&tx, &blockchain.lock().unwrap())
                    && validator::prevent_double_spend(&tx, &blockchain.lock().unwrap())
                {
                    blockchain.lock().unwrap().add_transaction(tx.clone());
                    println!("✅ Transaction added to pool.");
                } else {
                    println!("❌ Invalid transaction.");
                }
            }
            "3" => {
                let mut bc = blockchain.lock().unwrap();
                mining::mine_pending_transactions(&mut bc, &wallet.get_address(), &revstop);
            }
            "4" => {
                let blockchain = blockchain.lock().unwrap();
                let txs = blockchain.get_last_n_transactions(5);
                for tx in txs {
                    println!("{:?}", tx);
                }
            }
            "5" => {
                wallet.export_with_2fa();
            }
            "6" => {
                revstop.lock();
                println!("🔒 RevStop enabled.");
            }
            "7" => {
                if revstop.unlock() {
                    println!("🔓 RevStop disabled.");
                } else {
                    println!("❌ Incorrect password.");
                }
            }
            "8" => {
                println!("🆔 Wallet Address: {}", wallet.get_address());
            }
            "9" => {
                println!("Exiting...");
                std::process::exit(0);
            }
            _ => println!("Invalid choice."),
        }
    }
}