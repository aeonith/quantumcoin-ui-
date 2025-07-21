use crate::wallet::Wallet;
use crate::blockchain::Blockchain;
use crate::revstop::RevStop;
use std::io::{self, Write};

pub fn start_cli(wallet: &mut Wallet, blockchain: &mut Blockchain, revstop: &mut RevStop) {
    loop {
        println!("\n=== QuantumCoin CLI Menu ===");
        println!("1. View Wallet Balance");
        println!("2. Send Coins");
        println!("3. Mine Transactions");
        println!("4. Show Mining Progress");
        println!("5. Show RevStop Status");
        println!("6. Enable RevStop");
        println!("7. Disable RevStop");
        println!("8. Show Last 5 Transactions");
        println!("9. Export Wallet Backup with 2FA");
        println!("10. Show Wallet Address");
        println!("11. Exit");
        print!("Enter your choice: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim().parse::<u32>().unwrap_or(0);

        match choice {
            1 => {
                let balance = wallet.get_balance(blockchain);
                println!("🔐 Wallet Balance: {} QTC", balance);
            }
            2 => {
                print!("Enter recipient address: ");
                io::stdout().flush().unwrap();
                let mut to = String::new();
                io::stdin().read_line(&mut to).unwrap();

                print!("Enter amount to send: ");
                io::stdout().flush().unwrap();
                let mut amt_str = String::new();
                io::stdin().read_line(&mut amt_str).unwrap();
                let amount = amt_str.trim().parse::<u64>().unwrap_or(0);

                let tx = wallet.create_transaction(to.trim(), amount);
                blockchain.create_transaction(tx);
                println!("📤 Transaction queued successfully.");
            }
            3 => {
                blockchain.mine_pending_transactions(&wallet.get_address());
                println!("⛏️ Mined new block!");
            }
            4 => {
                println!("⛏️ Mining difficulty: {}", blockchain.difficulty);
                println!("🧱 Current block height: {}", blockchain.chain.len());
                println!("🎁 Block reward: {} QTC", blockchain.reward);
            }
            5 => {
                println!(
                    "🛡️ RevStop Status: {}",
                    if revstop.is_active() { "ENABLED" } else { "DISABLED" }
                );
            }
            6 => {
                revstop.enabled = true;
                if let Err(e) = revstop.save_status("revstop.json") {
                    println!("Failed to save RevStop status: {}", e);
                } else {
                    println!("🛡️ RevStop is now ENABLED");
                }
            }
            7 => {
                revstop.enabled = false;
                if let Err(e) = revstop.save_status("revstop.json") {
                    println!("Failed to save RevStop status: {}", e);
                } else {
                    println!("🛡️ RevStop is now DISABLED");
                }
            }
            8 => {
                let total = blockchain.chain.len();
                println!("🧾 Last 5 Transactions:");
                let mut count = 0;
                for block in blockchain.chain.iter().rev() {
                    for tx in block.transactions.iter().rev() {
                        if count >= 5 {
                            break;
                        }
                        println!(
                            "- [{}] {} ➡️ {} | {} QTC",
                            tx.id, tx.from, tx.to, tx.amount
                        );
                        count += 1;
                    }
                    if count >= 5 {
                        break;
                    }
                }
                if count == 0 {
                    println!("(No transactions found)");
                }
            }
            9 => {
                println!("📦 Exporting wallet with 2FA...");
                wallet.export_with_2fa();
            }
            10 => {
                println!("🔗 Wallet Address: {}", wallet.get_address());
            }
            11 => {
                println!("👋 Exiting QuantumCoin CLI. Goodbye.");
                break;
            }
            _ => println!("❌ Invalid option. Please enter 1–11."),
        }
    }
}