use crate::{blockchain::Blockchain, wallet::Wallet};
use std::io::{self, Write};

pub fn start_cli(wallet: &mut Wallet, blockchain: &mut Blockchain) {
    loop {
        print!("qcoin> ");
        io::stdout().flush().unwrap();
        let mut buf = String::new();
        if io::stdin().read_line(&mut buf).is_err() {
            println!("Error reading input"); continue;
        }
        let parts: Vec<_> = buf.trim().split_whitespace().collect();
        if parts.is_empty() { continue; }

        match parts[0] {
            "addr" => println!("📬 address: {}", wallet.get_address()),
            "bal"  => println!("💰 balance: {}", wallet.get_balance(blockchain)),
            "tx" if parts.len() == 3 => {
                if let Ok(amount) = parts[2].parse() {
                    let tx = wallet.create_transaction(parts[1], amount);
                    blockchain.pending_transactions.push(tx);
                    println!("✅ pending tx created");
                } else { println!("usage: tx <recipient> <amount>"); }
            }
            "mine" => {
                blockchain.mine_pending_transactions(wallet.get_address());
                println!("⛏️  block mined!");
            }
            "last" => wallet.show_last_transactions(blockchain),
            "export" => wallet.export_with_2fa(),
            "quit" | "exit" => break,
            _ => println!("commands: addr | bal | tx | mine | last | export | quit"),
        }
    }
}