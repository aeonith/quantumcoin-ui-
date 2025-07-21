use std::io::{self, Write};
use crate::wallet::Wallet;
use crate::revstop::RevStop;
use crate::blockchain::Blockchain;

pub fn launch_cli(wallet: &mut Wallet, blockchain: &mut Blockchain, revstop: &mut RevStop) {
    loop {
        println!("\n=== 🔐 QuantumCoin CLI Interface ===");
        println!("1. View Balance");
        println!("2. Send Coins");
        println!("3. Mine Pending Transactions");
        println!("4. Show Mining Progress");
        println!("5. Show RevStop Status");
        println!("6. Enable RevStop (Lock Wallet)");
        println!("7. Disable RevStop (Unlock Wallet)");
        println!("8. Show Last 5 Transactions");
        println!("9. Export Wallet Backup with 2FA");
        println!("10. Show Wallet Address");
        println!("11. Exit");

        print!("Select an option (1-11): ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        let choice = choice.trim();

        match choice {
            "1" => {
                let balance = wallet.get_balance(&blockchain);
                println!("💰 Balance: {} QTC", balance);
            }
            "2" => {
                print!("Enter recipient address: ");
                io::stdout().flush().unwrap();
                let mut recipient = String::new();
                io::stdin().read_line(&mut recipient).unwrap();

                print!("Enter amount to send: ");
                io::stdout().flush().unwrap();
                let mut amount = String::new();
                io::stdin().read_line(&mut amount).unwrap();

                let recipient = recipient.trim().to_string();
                let amount: f64 = amount.trim().parse().unwrap_or(0.0);

                if amount <= 0.0 {
                    println!("❌ Invalid amount.");
                    continue;
                }

                let tx = wallet.create_transaction(&recipient, amount);
                blockchain.add_transaction(tx);
                println!("✅ Transaction created!");
            }
            "3" => {
                blockchain.mine_pending_transactions(wallet);
                println!("⛏️ Mining complete.");
            }
            "4" => {
                blockchain.show_mining_progress();
            }
            "5" => {
                println!("{}", revstop.get_status_message());
            }
            "6" => {
                print!("Set a RevStop password to lock wallet: ");
                io::stdout().flush().unwrap();
                let mut password = String::new();
                io::stdin().read_line(&mut password).unwrap();
                revstop.lock(password.trim());
                println!("🔒 RevStop is now ACTIVE.");
            }
            "7" => {
                print!("Enter RevStop password to unlock: ");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let success = revstop.unlock(input.trim());
                if success {
                    println!("🔓 RevStop disabled successfully.");
                } else {
                    println!("❌ Incorrect password. RevStop remains active.");
                }
            }
            "8" => {
                wallet.show_last_transactions(&blockchain);
            }
            "9" => {
                wallet.export_with_2fa();
            }
            "10" => {
                println!("📬 Wallet Address: {}", wallet.get_address());
            }
            "11" => {
                println!("👋 Exiting CLI. Goodbye!");
                break;
            }
            _ => {
                println!("❌ Invalid selection. Choose a number between 1-11.");
            }
        }
    }
}