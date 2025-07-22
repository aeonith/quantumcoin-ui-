use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use crate::wallet::Wallet;
use crate::blockchain::Blockchain;
use crate::revstop::RevStop;
use crate::peer::broadcast_transaction;

pub fn run_cli(wallet: Arc<Mutex<Wallet>>, blockchain: Arc<Mutex<Blockchain>>, revstop: Arc<Mutex<RevStop>>) {
    loop {
        println!("\n=== QuantumCoin CLI ===");
        println!("1. Show Wallet Address");
        println!("2. Check Balance");
        println!("3. Send Coins");
        println!("4. Mine Transactions");
        println!("5. Show Mining Progress");
        println!("6. Show Last 5 Transactions");
        println!("7. Enable RevStop");
        println!("8. Disable RevStop");
        println!("9. Export Wallet (with 2FA)");
        println!("10. Exit");

        print!("Choose an option: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => {
                let wallet = wallet.lock().unwrap();
                println!("Wallet Address: {}", wallet.get_address());
            }
            "2" => {
                let bc = blockchain.lock().unwrap();
                let wallet = wallet.lock().unwrap();
                println!("Balance: {} QTC", bc.get_balance(&wallet.get_address()));
            }
            "3" => {
                let mut to = String::new();
                let mut amount = String::new();

                print!("Recipient Address: ");
                io::stdout().flush().unwrap();
                io::stdin().read_line(&mut to).unwrap();

                print!("Amount to Send: ");
                io::stdout().flush().unwrap();
                io::stdin().read_line(&mut amount).unwrap();

                let amount: f64 = match amount.trim().parse() {
                    Ok(a) => a,
                    Err(_) => {
                        println!("Invalid amount.");
                        continue;
                    }
                };

                let tx = {
                    let wallet = wallet.lock().unwrap();
                    wallet.create_transaction(to.trim(), amount)
                };

                {
                    let mut bc = blockchain.lock().unwrap();
                    bc.add_transaction(tx.clone());
                }

                broadcast_transaction(tx); // Peer-to-peer logic
                println!("Transaction sent & broadcasted!");
            }
            "4" => {
                let wallet = wallet.lock().unwrap();
                let mut bc = blockchain.lock().unwrap();
                bc.mine_pending_transactions(&wallet);
                println!("Block mined and rewards sent!");
            }
            "5" => {
                let bc = blockchain.lock().unwrap();
                bc.show_mining_progress();
            }
            "6" => {
                let bc = blockchain.lock().unwrap();
                let wallet = wallet.lock().unwrap();
                let txs = bc.get_last_n_transactions(&wallet.get_address(), 5);
                println!("Last 5 Transactions:");
                for tx in txs {
                    println!("{}", tx);
                }
            }
            "7" => {
                let mut rev = revstop.lock().unwrap();
                rev.lock();
                println!("RevStop is now enabled. Wallet frozen.");
            }
            "8" => {
                let mut password = String::new();
                print!("Enter RevStop password to unlock: ");
                io::stdout().flush().unwrap();
                io::stdin().read_line(&mut password).unwrap();

                let mut rev = revstop.lock().unwrap();
                if rev.unlock(password.trim()) {
                    println!("RevStop disabled. Wallet reactivated.");
                } else {
                    println!("Incorrect password.");
                }
            }
            "9" => {
                let mut auth_code = String::new();
                print!("Enter 2FA code: ");
                io::stdout().flush().unwrap();
                io::stdin().read_line(&mut auth_code).unwrap();

                let wallet = wallet.lock().unwrap();
                if wallet.export_with_2fa(auth_code.trim()) {
                    println!("Wallet exported successfully.");
                } else {
                    println!("2FA failed. Wallet not exported.");
                }
            }
            "10" => {
                println!("Exiting QuantumCoin CLI. Goodbye.");
                break;
            }
            _ => println!("Invalid option."),
        }
    }
}