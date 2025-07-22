mod wallet;
mod blockchain;
mod revstop;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use wallet::Wallet;
use blockchain::Blockchain;

#[derive(Serialize)]
struct BalanceResponse {
    balance: u64,
}

#[derive(Deserialize)]
struct SendRequest {
    recipient: String,
    amount: u64,
    password: String,
}

#[derive(Serialize)]
struct SendResponse {
    success: bool,
    message: String,
}

async fn get_balance(wallet: web::Data<Wallet>, blockchain: web::Data<Blockchain>) -> impl Responder {
    let balance = blockchain.get_balance(&wallet.get_address());
    HttpResponse::Ok().json(BalanceResponse { balance })
}

async fn send_coins(
    req: web::Json<SendRequest>,
    wallet: web::Data<Wallet>,
    blockchain: web::Data<Blockchain>,
) -> impl Responder {
    if !wallet.verify_password(&req.password) {
        return HttpResponse::Unauthorized().json(SendResponse {
            success: false,
            message: "Invalid 2FA password".to_string(),
        });
    }

    let tx = wallet.create_transaction(&req.recipient, req.amount);
    blockchain.add_transaction(tx);

    HttpResponse::Ok().json(SendResponse {
        success: true,
        message: "Transaction sent successfully".to_string(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🔐 Initializing QuantumCoin Node...");

    let wallet = Wallet::load_or_generate();
    let blockchain = Blockchain::load_from_file();

    println!("🚀 QuantumCoin Web Server Running at http://0.0.0.0:8080");

    let wallet_data = web::Data::new(wallet);
    let blockchain_data = web::Data::new(blockchain);

    HttpServer::new(move || {
        App::new()
            .app_data(wallet_data.clone())
            .app_data(blockchain_data.clone())
            .route("/balance", web::get().to(get_balance))
            .route("/send", web::post().to(send_coins))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}