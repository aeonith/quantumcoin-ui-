#[macro_use] extern crate rocket;

use rocket::fs::{FileServer, relative};
use rocket::serde::json::Json;
use rocket::State;
use std::sync::Mutex;

mod wallet;
mod blockchain;
mod revstop;
mod transaction;

use wallet::Wallet;
use blockchain::Blockchain;
use transaction::Transaction;

struct AppState {
    blockchain: Mutex<Blockchain>,
    wallet: Mutex<Wallet>,
}

// Health check route
#[get("/")]
fn index() -> &'static str {
    "🚀 QuantumCoin API is running!"
}

// Get wallet address
#[get("/wallet/address")]
fn get_address(state: &State<AppState>) -> String {
    let wallet = state.wallet.lock().unwrap();
    wallet.get_address()
}

// Get wallet balance
#[get("/wallet/balance")]
fn get_balance(state: &State<AppState>) -> String {
    let wallet = state.wallet.lock().unwrap();
    format!("{:.6}", wallet.balance)
}

// Send transaction
#[post("/wallet/send", format = "json", data = "<tx>")]
fn send_transaction(state: &State<AppState>, tx: Json<Transaction>) -> String {
    let mut blockchain = state.blockchain.lock().unwrap();
    blockchain.add_transaction(tx.into_inner());
    "Transaction added".to_string()
}

// Mine pending transactions
#[post("/mine")]
fn mine_block(state: &State<AppState>) -> String {
    let mut blockchain = state.blockchain.lock().unwrap();
    blockchain.mine_pending_transactions();
    "Block mined".to_string()
}

#[launch]
fn rocket() -> _ {
    let wallet = Wallet::load_or_generate("wallet_public.key", "wallet_private.key");
    let blockchain = Blockchain::load_or_initialize(&wallet.get_address());

    rocket::build()
        .manage(AppState {
            wallet: Mutex::new(wallet),
            blockchain: Mutex::new(blockchain),
        })
        .mount("/", routes![
            index,
            get_address,
            get_balance,
            send_transaction,
            mine_block,
        ])
        .mount("/static", FileServer::from(relative!("static")))
}