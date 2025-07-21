use std::env;
use crate::{Wallet, Blockchain};
use crate::transaction::Transaction;

pub fn run() {
    let mut args = env::args().skip(1);
    let cmd = args.next().unwrap_or_default();

    // load or create
    let mut wallet = Wallet::generate();
    let mut chain  = Blockchain::new();

    match cmd.as_str() {
        "balance" => {
            println!("💰 Balance: {}", wallet.balance(&chain));
        }
        "send" => {
            let to     = args.next().expect("recipient required");
            let amount = args.next().expect("amount required").parse().expect("invalid amount");
            let tx = wallet.create_transaction(&to, amount);
            chain.add_block(vec![tx.clone()]);
            println!("➤ Sent {} to {} (tx_id={})", amount, to, tx.id);
        }
        "mine" => {
            chain.add_block(vec![]);
            let b = chain.chain.last().unwrap();
            println!("⛏️  Mined block #{} with hash {}", b.index, b.hash);
        }
        "chain" => {
            for b in &chain.chain {
                println!("--- Block #{} @{} ---", b.index, b.timestamp);
                for tx in &b.transactions {
                    println!("  • {} -> {} : {}", tx.from, tx.to, tx.amount);
                }
            }
        }
        _ => {
            eprintln!("Usage:");
            eprintln!("  quantumcoin balance");
            eprintln!("  quantumcoin send <to> <amount>");
            eprintln!("  quantumcoin mine");
            eprintln!("  quantumcoin chain");
        }
    }
}