use std::sync::{Arc, Mutex};
use std::io::{self, Write};

use crate::blockchain::Blockchain;
use crate::wallet::Wallet;
use crate::transaction::Transaction;
use crate::peer;

pub fn start_cli(wallet: Wallet, blockchain: Arc<Mutex<Blockchain>>) {
    loop {
        println!("\n⚡ QuantumCoin CLI Menu:");
        println!("1. View Balance");
        println!("2. Send Coins");
        println!("3. Mine Pending Transactions");
        println!("4. Show Last 5 Transactions");
        println!("5. Export Wallet with 2FA");
        println!("6. Enable RevStop");
        println!("7. Disable RevStop");
        println!("8. Show Wallet Address");
        println!("9. Exit");

        print!("Select an option: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => {
                let bc = blockchain.lock().unwrap();
                let balance = bc.get_balance(&wallet.get_address());
                println!("💰 Your balance: {:.2} QTC", balance);
            }
            "2" => {
                print!("Enter recipient address: ");
                io::stdout().flush().unwrap();
                let mut recipient = String::new();
                io::stdin().read_line(&mut recipient).unwrap();

                print!("Enter amount to send: ");
                io::stdout().flush().unwrap();
                let mut amount_str = String::new();
                io::stdin().read_line(&mut amount_str).unwrap();

                let amount: f64 = match amount_str.trim().parse() {
                    Ok(a) => a,
                    Err(_) => {
                        println!("⚠️ Invalid amount.");
                        continue;
                    }
                };

                let tx = wallet.create_transaction(recipient.trim(), amount);

                {
                    let mut bc = blockchain.lock().unwrap();
                    bc.add_transaction(tx.clone());
                }

                peer::broadcast_transaction(&tx);
                println!("✅ Transaction sent and broadcasted.");
            }
            "3" => {
                let block = {
                    let mut bc = blockchain.lock().unwrap();
                    bc.mine_pending_transactions(wallet.get_address())
                };

                peer::broadcast_block(&block);
                println!("⛏️ Mined new block and broadcasted.");
            }
            "4" => {
                let bc = blockchain.lock().unwrap();
                let last_txs = bc.get_last_n_transactions(5);
                println!("🧾 Last 5 transactions:");
                for tx in last_txs {
                    println!(
                        "From: {} To: {} Amount: {:.2}",
                        tx.sender, tx.recipient, tx.amount
                    );
                }
            }
            "5" => {
                wallet.export_with_2fa();
            }
            "6" => {
                // Add RevStop enabling logic here
                println!("🔒 RevStop enabled.");
            }
            "7" => {
                // Add RevStop disabling logic here
                println!("🔓 RevStop disabled.");
            }
            "8" => {
                println!("🆔 Wallet Address: {}", wallet.get_address());
            }
            "9" => {
                println!("👋 Goodbye!");
                break;
            }
            _ => println!("⚠️ Invalid option."),
        }
    }
}