use crate::wallet::Wallet;
use crate::blockchain::Blockchain;
use crate::revstop::RevStop;
use std::io::{self, Write};

pub fn run_cli(wallet: &mut Wallet, blockchain: &mut Blockchain, revstop: &mut RevStop) {
    loop {
        println!("\n=== QuantumCoin CLI ===");
        println!("1. View Balance");
        println!("2. Send Coins");
        println!("3. Mine Transactions");
        println!("4. Show Mining Progress");
        println!("5. Show Last 5 Transactions");
        println!("6. Show Wallet Address");
        println!("7. Enable RevStop");
        println!("8. Disable RevStop");
        println!("9. Export Wallet (2FA)");
        println!("0. Exit");

        print!("Choose an option: ");
        io::stdout().flush().unwrap();
        let mut option = String::new();
        io::stdin().read_line(&mut option).unwrap();

        match option.trim() {
            "1" => {
                let balance = wallet.get_balance(blockchain);
                println!("💰 Balance: {:.4} QTC", balance);
            }
            "2" => {
                print!("Enter recipient address: ");
                io::stdout().flush().unwrap();
                let mut recipient = String::new();
                io::stdin().read_line(&mut recipient).unwrap();

                print!("Enter amount: ");
                io::stdout().flush().unwrap();
                let mut amount_str = String::new();
                io::stdin().read_line(&mut amount_str).unwrap();
                let amount: f64 = amount_str.trim().parse().unwrap_or(0.0);

                if revstop.status() {
                    println!("❌ RevStop is active. Unlock to send coins.");
                } else {
                    let tx = wallet.create_transaction(&recipient.trim(), amount);
                    blockchain.add_transaction(tx);
                    println!("✅ Transaction added to mempool.");
                }
            }
            "3" => {
                blockchain.mine_pending_transactions(wallet);
                println!("⛏️ Mining complete.");
            }
            "4" => {
                blockchain.show_mining_progress();
            }
            "5" => {
                wallet.show_last_transactions(blockchain);
            }
            "6" => {
                println!("📬 Your Wallet Address:\n{}", wallet.get_address());
            }
            "7" => {
                revstop.lock();
                println!("🔒 RevStop is now ENABLED.");
            }
            "8" => {
                print!("Enter RevStop password to unlock: ");
                io::stdout().flush().unwrap();
                let mut password = String::new();
                io::stdin().read_line(&mut password).unwrap();
                if revstop.unlock(password.trim()) {
                    println!("🔓 RevStop DISABLED.");
                } else {
                    println!("❌ Incorrect password. RevStop remains ACTIVE.");
                }
            }
            "9" => {
                wallet.export_with_2fa();
            }
            "0" => {
                println!("👋 Exiting. Goodbye.");
                break;
            }
            _ => println!("❓ Invalid option. Try again."),
        }
    }
}