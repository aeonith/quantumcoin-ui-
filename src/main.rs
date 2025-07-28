#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod blockchain;
mod routes;
mod wallet;

use blockchain::Blockchain;
use routes::{get_balance, get_wallet_keys, mine_block, send_transaction};
use std::sync::Mutex;
use wallet::Wallet;

fn main() {
    // Load or create wallet
    let wallet = Wallet::load_from_files("wallet_public.key", "wallet_private.key")
        .unwrap_or_else(|_| {
            let new_wallet = Wallet::new();
            new_wallet.save_to_files("wallet_public.key", "wallet_private.key").unwrap();
            new_wallet
        });

    // Load or create blockchain
    let blockchain = Blockchain::load_from_disk("blockchain.json")
        .unwrap_or_else(|| Blockchain::new());

    // Launch Rocket server
    rocket::ignite()
        .manage(Mutex::new(wallet))
        .manage(Mutex::new(blockchain))
        .mount(
            "/",
            routes![get_wallet_keys, send_transaction, mine_block, get_balance],
        )
        .launch();
}