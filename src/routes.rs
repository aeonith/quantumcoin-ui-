use crate::blockchain::Blockchain;
use crate::transaction::Transaction;
use crate::wallet::Wallet;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::State;
use std::sync::Arc;
use rocket::tokio::sync::Mutex;

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct SendRequest {
    pub recipient: String,
    pub amount: f64,
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct WalletInfo {
    pub address: String,
    pub balance: f64,
}

#[get("/wallet")]
pub async fn get_wallet(wallet: &State<Arc<Mutex<Wallet>>>, blockchain: &State<Arc<Mutex<Blockchain>>>) -> Json<WalletInfo> {
    let wallet = wallet.lock().await;
    let blockchain = blockchain.lock().await;

    Json(WalletInfo {
        address: wallet.get_address_base64(),
        balance: blockchain.get_balance(&wallet.get_address_base64()),
    })
}

#[post("/send", format = "json", data = "<req>")]
pub async fn send_coins(
    req: Json<SendRequest>,
    wallet: &State<Arc<Mutex<Wallet>>>,
    blockchain: &State<Arc<Mutex<Blockchain>>>,
) -> &'static str {
    let wallet = wallet.lock().await;
    let mut blockchain = blockchain.lock().await;

    let sender_address = wallet.get_address_base64();
    let tx = wallet.create_transaction(&req.recipient, req.amount, &sender_address);
    blockchain.add_transaction(tx);

    "Transaction added"
}

#[post("/mine")]
pub async fn mine(
    wallet: &State<Arc<Mutex<Wallet>>>,
    blockchain: &State<Arc<Mutex<Blockchain>>>,
) -> &'static str {
    let wallet = wallet.lock().await;
    let mut blockchain = blockchain.lock().await;

    let miner_address = wallet.get_address_base64();
    blockchain.mine_pending_transactions(&miner_address);

    "Block mined"
}

pub fn routes() -> Vec<rocket::Route> {
    routes![get_wallet, send_coins, mine]
}