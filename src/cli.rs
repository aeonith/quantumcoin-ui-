use crate::{blockchain::Blockchain, revstop::RevStop, wallet::Wallet};
use std::io::{self, Write};

pub fn start_cli(wallet: &mut Wallet, bc: &mut Blockchain, rs: &mut RevStop) {
    loop {
        print!("qcoin> ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() { continue; }
        match line.trim().split_whitespace().collect::<Vec<_>>().as_slice() {
            ["addr"]     => println!("Address: {}", wallet.get_address()),
            ["bal"]      => println!("Balance: {}", wallet.get_balance(bc)),
            ["tx", to, x] if x.parse::<u64>().is_ok() => {
                let tx = wallet.create_transaction(to, x.parse().unwrap());
                bc.create_transaction(tx);
                println!("Queued TX");
            }
            ["mine"]     => { bc.mine_pending_transactions(&wallet.get_address()); println!("Mined"); }
            ["rev"]      => println!("RevStop: {}", rs.is_active()),
            ["lock"]     => { rs.enabled = true; rs.save_status("revstop.json").ok(); println!("Locked"); }
            ["unlock"]   => { rs.enabled = false; rs.save_status("revstop.json").ok(); println!("Unlocked"); }
            ["kyc", c]   => println!("KYC {}", if crate::kyc::verify(&wallet.get_address(), c) { "OK" } else { "FAIL" }),
            ["exit"]     => break,
            _            => println!("Commands: addr, bal, tx <to> <amt>, mine, rev, lock, unlock, kyc <code>, exit"),
        }
    }
}