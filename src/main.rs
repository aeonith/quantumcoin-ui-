use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use base64::{engine::general_purpose, Engine as _};

fn get_wallet_address() -> String {
    // Simulate a hardcoded base64-encoded Dilithium public key
    let pub_key_bytes = b"MyQuantumCoinPublicKey1234567890"; // Replace with actual key if needed
    general_purpose::STANDARD.encode(pub_key_bytes)
}

async fn wallet_handler() -> impl Responder {
    let wallet_address = get_wallet_address();
    HttpResponse::Ok().body(format!("QuantumCoin Wallet Address:\n{}", wallet_address))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 QuantumCoin Web Server Started on port 8080");

    HttpServer::new(|| {
        App::new()
            .route("/wallet", web::get().to(wallet_handler))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}