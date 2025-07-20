use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Deserialize)]
struct TxReq {
    sender: String,
    recipient: String,
    amount: f64,
}

#[derive(Serialize)]
struct ApiMsg<'a> {
    message: &'a str,
}

#[post("/api/send")]
async fn send(tx: web::Json<TxReq>) -> impl Responder {
    println!(
        "TX: {} → {} amount {}",
        tx.sender, tx.recipient, tx.amount
    );
    HttpResponse::Ok().json(ApiMsg { message: "tx accepted" })
}

#[get("/api/balance/{wallet}")]
async fn balance(wallet: web::Path<String>) -> impl Responder {
    // fake balance until we wire real storage
    HttpResponse::Ok().json(serde_json::json!({
        "wallet": wallet.into_inner(),
        "balance": 0.0
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    // Render sets $PORT; default to 8080 locally
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8080".into())
        .parse()
        .expect("PORT must be a number");
    println!("✨ Backend running on port {port}");
    HttpServer::new(|| App::new().service(send).service(balance))
        .bind(("0.0.0.0", port))?
        .run()
        .await
}