mod wallet;
mod transaction;
mod blockchain;
mod revstop;

use wallet::Wallet;
use transaction::{Transaction, TransactionType};
use blockchain::Blockchain;
use revstop::RevStop;
use std::sync::{Arc, Mutex};
use std::io::{self, Write};

fn calculate_price(supply: f64, demand: f64) -> f64 {
    let mut price = (demand / (supply + 1.0)) * 10.0;
    if price < 0.25 { price = 0.25; }
    price
}

fn main() {
    println!("🚀 QuantumCoin Engine Ready");

    let wallet = Wallet::load_or_create("wallet.json");
    let revstop = RevStop::load_status();
    let blockchain = Arc::new(Mutex::new(Blockchain::load_from_file("blockchain.json")));

    let initial_supply = blockchain.lock().unwrap().total_supply();
    let mut total_demand = 0.0;

    loop {
        println!("\n📜 Menu:\n1. Balance\n2. Buy\n3. Sell\n4. Mine\n5. Exit");
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "1" => {
                println!("🔐 Address: {}", wallet.address());
                println!("💰 Balance: {} QTC", blockchain.lock().unwrap().get_balance(&wallet.address()));
            }
            "2" => {
                let price = calculate_price(initial_supply, total_demand);
                println!("✅ Current price: ${:.2}", price);
                println!("How much do you want to buy?");
                let mut amt = String::new();
                io::stdin().read_line(&mut amt).unwrap();
                let coins = amt.trim().parse::<f64>().unwrap_or(0.0);
                let cost = coins * price;
                println!("💵 This will cost ${:.2}. Confirm? (y/n)", cost);
                let mut confirm = String::new();
                io::stdin().read_line(&mut confirm).unwrap();
                if confirm.trim() == "y" {
                    let tx = wallet.create_transaction("network".to_string(), coins, TransactionType::Buy);
                    blockchain.lock().unwrap().add_transaction(tx);
                    total_demand += coins;
                    println!("✅ Buy order submitted.");
                }
            }
            "3" => {
                println!("How much do you want to sell?");
                let mut amt = String::new();
                io::stdin().read_line(&mut amt).unwrap();
                let coins = amt.trim().parse::<f64>().unwrap_or(0.0);
                let tx = wallet.create_transaction("market".to_string(), coins, TransactionType::Sell);
                blockchain.lock().unwrap().add_transaction(tx);
                println!("✅ Sell order submitted.");
            }
            "4" => {
                blockchain.lock().unwrap().mine_pending_transactions(wallet.address());
                println!("⛏️ Block mined and reward granted.");
            }
            "5" => break,
            _ => println!("❌ Invalid input."),
        }
    }

    blockchain.lock().unwrap().save_to_file("blockchain.json");
    RevStop::save_status(revstop);
}