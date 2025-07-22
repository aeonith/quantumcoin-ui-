use actix_web::{post, get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::sync::Mutex;
use uuid::Uuid;

use crate::wallet::{Wallet, load_wallet_from_file, save_wallet_to_file};
use crate::twfa::{generate_secret, verify_token};
use crate::blockchain::{get_current_price, send_qtc_to_wallet};

#[derive(Deserialize)]
pub struct CreateAccountRequest {
    username: String,
    password: String,
    accepted_terms: bool,
}

#[derive(Serialize)]
pub struct CreateAccountResponse {
    status: String,
    wallet_address: String,
    qr_code: String,
    secret: String,
}

#[post("/create_account")]
pub async fn create_account(data: web::Json<CreateAccountRequest>) -> impl Responder {
    if !data.accepted_terms {
        return HttpResponse::BadRequest().json("Must accept Terms & Conditions to create account.");
    }

    let wallet = Wallet::new(&data.username, &data.password);
    let address = wallet.get_address();
    let secret = generate_secret(&data.username);
    let qr = format!("otpauth://totp/QuantumCoin:{}?secret={}&issuer=QuantumCoin", data.username, secret);

    save_wallet_to_file(&wallet);

    HttpResponse::Ok().json(CreateAccountResponse {
        status: "success".to_string(),
        wallet_address: address,
        qr_code: qr,
        secret,
    })
}

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
    token: String,
}

#[post("/login")]
pub async fn login(req: web::Json<LoginRequest>) -> impl Responder {
    let wallet = match load_wallet_from_file(&req.username) {
        Some(w) => w,
        None => return HttpResponse::Unauthorized().body("Invalid credentials."),
    };

    if !wallet.verify_password(&req.password) {
        return HttpResponse::Unauthorized().body("Invalid password.");
    }

    if !verify_token(&req.username, &req.token) {
        return HttpResponse::Unauthorized().body("Invalid 2FA token.");
    }

    HttpResponse::Ok().json(wallet.get_address())
}

#[derive(Deserialize)]
pub struct WebhookPayload {
    btc_txid: String,
    btc_amount: f64,
    from_btc_address: String,
}

#[post("/btc_webhook")]
pub async fn btc_webhook(payload: web::Json<WebhookPayload>) -> impl Responder {
    let btc_usd_price = match get_current_price("bitcoin").await {
        Ok(p) => p,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to fetch BTC price"),
    };

    let received_usd = payload.btc_amount * btc_usd_price;
    let qtc_to_send = (received_usd / 0.25).floor() as u64;

    let mapped_wallet = map_btc_address_to_wallet(&payload.from_btc_address);
    if let Some(qtc_wallet) = mapped_wallet {
        match send_qtc_to_wallet(&qtc_wallet, qtc_to_send) {
            Ok(_) => HttpResponse::Ok().json(format!("Sent {} QTC to {}", qtc_to_send, qtc_wallet)),
            Err(_) => HttpResponse::InternalServerError().body("Dispatch failed"),
        }
    } else {
        HttpResponse::BadRequest().body("BTC address not linked to any QuantumCoin wallet")
    }
}

fn map_btc_address_to_wallet(btc_addr: &str) -> Option<String> {
    let db = fs::read_to_string("btc_to_qtc_map.json").unwrap_or("{}".to_string());
    let map: HashMap<String, String> = serde_json::from_str(&db).unwrap_or_default();
    map.get(btc_addr).cloned()
}