use std::io::{self, Write};
use crate::wallet::Wallet;
use crate::revstop::{RevStop, enable_revstop, disable_revstop, get_revstop_status};
use crate::blockchain::Blockchain;

pub fn cli_menu(wallet: &mut Wallet, blockchain: &mut Blockchain, revstop: &mut RevStop) {
    loop {
        println!("\n=== QuantumCoin CLI Wallet ===");
        println!("1. Check Balance");
        println!("2. Send Coins");
        println!("3. Mine Transactions");
        println!("4. Show Mining Progress");
        println!("5. Show RevStop Status");
        println!("6. Enable RevStop");
        println!("7. Disable RevStop");
        println!("8. Show Last 5 Transactions");
        println!("9. Export Wallet with 2FA");
        println!("10. Show Wallet Address");
        println!("11. Exit");

        print!("Select an option: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => {
                let balance = wallet.get_balance(&blockchain);
                println!("💰 Current Balance: {} QTC", balance);
            }
            "2" => {
                print!("Recipient Address: ");
                io::stdout().flush().unwrap();
                let mut recipient = String::new();
                io::stdin().read_line(&mut recipient).unwrap();
                let recipient = recipient.trim().to_string();

                print!("Amount to Send: ");
                io::stdout().flush().unwrap();
                let mut amount = String::new();
                io::stdin().read_line(&mut amount).unwrap();
                let amount: f64 = amount.trim().parse().unwrap_or(0.0);

                let tx = wallet.create_transaction(&recipient, amount);
                blockchain.add_transaction(tx);
                println!("✅ Transaction created and added to mempool.");
            }
            "3" => {
                blockchain.mine_pending_transactions(wallet);
                println!("🪨 Mining complete.");
            }
            "4" => {
                blockchain.show_mining_progress();
            }
            "5" => {
                let status = get_revstop_status(revstop);
                println!("🔐 RevStop is currently: {}", if status { "ENABLED" } else { "DISABLED" });
            }
            "6" => {
                enable_revstop(revstop);
            }
            "7" => {
                disable_revstop(revstop);
            }
            "8" => {
                blockchain.show_last_transactions();
            }
            "9" => {
                wallet.export_with_2fa();
            }
            "10" => {
                let address = wallet.get_address();
                println!("📬 Wallet Address: {}", address);
            }
            "11" => {
                println!("👋 Exiting QuantumCoin CLI. Goodbye.");
                break;
            }
            _ => println!("❌ Invalid option. Try again."),
        }
    }
}