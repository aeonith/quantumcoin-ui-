use actix_web::{get, post, web, HttpResponse, Responder};
use serde_json::json;

use crate::{wallet::*, blockchain::*, revstop::*};

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({"status": "QuantumCoin API is live"}))
}

#[post("/wallet/create")]
async fn create_wallet() -> impl Responder {
    let wallet = Wallet::new();
    wallet.save_to_files().unwrap();
    HttpResponse::Ok().json(wallet.get_address())
}

#[get("/wallet/balance")]
async fn get_balance() -> impl Responder {
    match Wallet::load_from_files() {
        Ok(wallet) => HttpResponse::Ok().json(wallet.get_balance()),
        Err(_) => HttpResponse::InternalServerError().body("Wallet not found"),
    }
}

#[post("/transaction/send")]
async fn send_transaction(req: web::Json<(String, f64)>) -> impl Responder {
    let (recipient, amount) = req.into_inner();
    match Wallet::load_from_files() {
        Ok(wallet) => {
            let tx = wallet.create_transaction(&recipient, amount);
            HttpResponse::Ok().json(tx)
        }
        Err(_) => HttpResponse::InternalServerError().body("Transaction failed"),
    }
}

#[post("/mine")]
async fn mine_block() -> impl Responder {
    let mut blockchain = Blockchain::load_from_file().unwrap();
    blockchain.mine_pending_transactions();
    blockchain.save_to_file().unwrap();
    HttpResponse::Ok().json("Block mined")
}

#[get("/price")]
async fn get_price() -> impl Responder {
    // In production, query a real price feed or calculate supply/demand
    HttpResponse::Ok().json(json!({
        "price": 749.23,
        "change": 14.36,
        "percent": 1.98
    }))
}

#[get("/transactions")]
async fn get_last_transactions() -> impl Responder {
    let blockchain = Blockchain::load_from_file().unwrap();
    let txs = blockchain.get_last_n_transactions(5);
    HttpResponse::Ok().json(txs)
}

#[get("/revstop/status")]
async fn get_revstop_status() -> impl Responder {
    let status = load_status();
    HttpResponse::Ok().json(json!({ "revstop_enabled": status }))
}

#[post("/revstop/lock")]
async fn lock_revstop() -> impl Responder {
    if lock().is_ok() {
        HttpResponse::Ok().body("RevStop locked")
    } else {
        HttpResponse::InternalServerError().body("Lock failed")
    }
}

#[post("/revstop/unlock")]
async fn unlock_revstop(req: web::Json<String>) -> impl Responder {
    let password = req.into_inner();
    match unlock(&password) {
        Ok(_) => HttpResponse::Ok().body("Unlocked"),
        Err(_) => HttpResponse::Unauthorized().body("Invalid password"),
    }
}