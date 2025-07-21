use std::io::{self, Write};
use crate::wallet::Wallet;
use crate::revstop::RevStop;
use crate::blockchain::{mine_pending_transactions, show_mining_progress};

pub fn run_cli(wallet: &mut Wallet, revstop: &mut RevStop) {
    loop {
        println!("\n================ QuantumCoin CLI ================");
        println!("1. 🔍 View Balance");
        println!("2. 💸 Send Coins");
        println!("3. ⛏️  Mine Transactions");
        println!("4. 📊 Show Mining Progress");
        println!("5. 🔒 RevStop Status");
        println!("6. 🔐 Enable RevStop");
        println!("7. 🔓 Disable RevStop");
        println!("8. 📜 Show Last 5 Transactions");
        println!("9. 🧾 Export Wallet (2FA)");
        println!("10. 🏦 Show Wallet Address");
        println!("11. ❌ Exit");
        print!("Enter your choice: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        match input.trim() {
            "1" => {
                println!("💰 Balance: {} QTC", wallet.get_balance());
            },
            "2" => {
                print!("Enter recipient address: ");
                io::stdout().flush().unwrap();
                let mut to = String::new();
                io::stdin().read_line(&mut to).unwrap();

                print!("Enter amount to send: ");
                io::stdout().flush().unwrap();
                let mut amount_str = String::new();
                io::stdin().read_line(&mut amount_str).unwrap();

                if let Ok(amount) = amount_str.trim().parse::<u64>() {
                    match wallet.create_transaction(to.trim().to_string(), amount) {
                        Ok(_) => println!("✅ Transaction created!"),
                        Err(e) => println!("❌ Error: {}", e),
                    }
                } else {
                    println!("❌ Invalid amount.");
                }
            },
            "3" => {
                mine_pending_transactions(wallet.address());
                println!("⛏️ Mining complete.");
            },
            "4" => {
                show_mining_progress();
            },
            "5" => {
                println!("{}", revstop.status_string());
            },
            "6" => {
                revstop.lock();
                println!("🔐 RevStop enabled.");
            },
            "7" => {
                print!("Enter your RevStop password to unlock: ");
                io::stdout().flush().unwrap();
                let mut pass = String::new();
                io::stdin().read_line(&mut pass).unwrap();

                if revstop.unlock(pass.trim()) {
                    println!("✅ RevStop disabled.");
                } else {
                    println!("❌ Incorrect password.");
                }
            },
            "8" => {
                wallet.show_last_transactions();
            },
            "9" => {
                match wallet.export_with_2fa() {
                    Ok(_) => println!("✅ Wallet exported with 2FA."),
                    Err(e) => println!("❌ Export failed: {}", e),
                }
            },
            "10" => {
                println!("🏦 Wallet Address: {}", wallet.address());
            },
            "11" => {
                println!("👋 Exiting CLI.");
                break;
            },
            _ => println!("❌ Invalid choice."),
        }
    }
}