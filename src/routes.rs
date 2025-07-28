use rocket::{get, post, State};
use rocket_contrib::json::Json;
use serde::Deserialize;
use std::sync::Mutex;
use crate::blockchain::{Blockchain, Transaction};
use crate::wallet::Wallet;

#[derive(Deserialize)]
pub struct TransactionRequest {
    pub recipient: String,
    pub amount: u64,
}

#[get("/wallet")]
pub fn get_wallet_keys(wallet_data: State<Mutex<Wallet>>) -> Json<serde_json::Value> {
    let wallet = wallet_data.lock().unwrap();
    Json(serde_json::json!({
        "publicKey": wallet.get_public_key_base64(),
        "privateKey": wallet.get_private_key_base64(),
    }))
}

#[post("/send", format = "json", data = "<req>")]
pub fn send_transaction(
    req: Json<TransactionRequest>,
    wallet_data: State<Mutex<Wallet>>,
    blockchain_data: State<Mutex<Blockchain>>,
) -> Json<serde_json::Value> {
    let wallet = wallet_data.lock().unwrap();
    let sender_address = wallet.get_address_base64();
    let tx = Transaction::new(&sender_address, &req.recipient, req.amount, &wallet);
    let mut blockchain = blockchain_data.lock().unwrap();
    blockchain.add_transaction(tx);
    Json(serde_json::json!({"status": "Transaction added"}))
}

#[post("/mine")]
pub fn mine_block(
    wallet_data: State<Mutex<Wallet>>,
    blockchain_data: State<Mutex<Blockchain>>,
) -> Json<serde_json::Value> {
    let wallet = wallet_data.lock().unwrap();
    let miner_address = wallet.get_address_base64();
    let mut blockchain = blockchain_data.lock().unwrap();
    blockchain.mine_pending_transactions(&miner_address);
    Json(serde_json::json!({"status": "Block mined"}))
}

#[get("/balance")]
pub fn get_balance(
    wallet_data: State<Mutex<Wallet>>,
    blockchain_data: State<Mutex<Blockchain>>,
) -> Json<serde_json::Value> {
    let wallet = wallet_data.lock().unwrap();
    let blockchain = blockchain_data.lock().unwrap();
    let balance = blockchain.get_balance(&wallet.get_address_base64());
    Json(serde_json::json!({ "balance": balance }))
}