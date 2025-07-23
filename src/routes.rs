use crate::blockchain::Blockchain;
use crate::wallet::Wallet;
use crate::revstop;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Serialize)]
struct BalanceResponse {
    address: String,
    balance: u64,
}

#[derive(Deserialize)]
struct SendRequest {
    to: String,
    amount: u64,
}

#[get("/balance")]
async fn balance(
    blockchain: web::Data<Arc<Mutex<Blockchain>>>,
    wallet: web::Data<Arc<Mutex<Wallet>>>,
) -> impl Responder {
    let chain = blockchain.lock().unwrap();
    let wallet = wallet.lock().unwrap();
    let balance = chain.get_balance(&wallet.public_key);
    HttpResponse::Ok().json(BalanceResponse {
        address: wallet.public_key.clone(),
        balance,
    })
}

#[get("/address")]
async fn address(wallet: web::Data<Arc<Mutex<Wallet>>>) -> impl Responder {
    let wallet = wallet.lock().unwrap();
    HttpResponse::Ok().body(wallet.get_address())
}

#[post("/send")]
async fn send(
    blockchain: web::Data<Arc<Mutex<Blockchain>>>,
    wallet: web::Data<Arc<Mutex<Wallet>>>,
    req: web::Json<SendRequest>,
) -> impl Responder {
    let mut chain = blockchain.lock().unwrap();
    let wallet = wallet.lock().unwrap();
    if req.amount == 0 {
        return HttpResponse::BadRequest().body("Amount must be greater than 0");
    }
    let tx = chain.create_transaction(&wallet.public_key, &req.to, req.amount);
    chain.add_transaction(tx);
    HttpResponse::Ok().body("Transaction submitted")
}

#[post("/mine")]
async fn mine(blockchain: web::Data<Arc<Mutex<Blockchain>>>) -> impl Responder {
    let mut chain = blockchain.lock().unwrap();
    chain.mine_pending_transactions();
    HttpResponse::Ok().body("Block mined")
}

#[get("/revstop")]
async fn revstop_status(wallet: web::Data<Arc<Mutex<Wallet>>>) -> impl Responder {
    let wallet = wallet.lock().unwrap();
    let status = revstop::get_revstop_status(&wallet.public_key);
    HttpResponse::Ok().body(format!("RevStop: {}", status))
}

#[post("/export")]
async fn export(
    wallet: web::Data<Arc<Mutex<Wallet>>>,
    body: web::Json<serde_json::Value>,
) -> impl Responder {
    let password = body["password"].as_str().unwrap_or_default();
    let wallet = wallet.lock().unwrap();
    match wallet.export_with_2fa(password) {
        Ok(_) => HttpResponse::Ok().body("Wallet exported with 2FA"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

pub fn get_routes() -> Vec<actix_web::Route> {
    actix_web::web::scope("")
        .service(balance)
        .service(address)
        .service(send)
        .service(mine)
        .service(revstop_status)
        .service(export)
        .into_inner()
        .routes
}