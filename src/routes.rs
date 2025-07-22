use actix_web::{web, HttpResponse};
use crate::{wallet::Wallet, blockchain::Blockchain};
use std::sync::Mutex;
use serde::Deserialize;
use otpauth::TOTP;
use qrcode::{QrCode, render::unicode};

#[derive(Deserialize)]
pub struct TwoFASetup {
    pub password: String,
}

#[derive(Deserialize)]
pub struct TwoFACheck {
    pub code: String,
    pub password: String,
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/setup-2fa").route(web::post().to(setup_2fa)))
       .service(web::resource("/verify-2fa").route(web::post().to(verify_2fa)));
}

async fn setup_2fa(wallet: web::Data<Mutex<Wallet>>, req: web::Json<TwoFASetup>) -> HttpResponse {
    let mut w = wallet.lock().unwrap();
    if !w.verify_password(&req.password) {
        return HttpResponse::Unauthorized().body("Wrong password");
    }

    let secret = TOTP::generate_secret();
    w.enable_2fa(secret.clone());

    let qr_url = TOTP::from_base32(&secret).unwrap().get_url("QuantumCoin", "user@quantumcoin.com");
    let qr = QrCode::new(qr_url.clone()).unwrap();
    let image = qr.render::<unicode::Dense1x2>().build();

    HttpResponse::Ok().json(serde_json::json!({
        "secret": secret,
        "qr_url": qr_url,
        "qr_ascii": image
    }))
}

async fn verify_2fa(wallet: web::Data<Mutex<Wallet>>, req: web::Json<TwoFACheck>) -> HttpResponse {
    let w = wallet.lock().unwrap();
    if !w.verify_password(&req.password) {
        return HttpResponse::Unauthorized().body("Bad password");
    }

    if w.verify_2fa(&req.code) {
        HttpResponse::Ok().body("✅ 2FA Verified")
    } else {
        HttpResponse::Unauthorized().body("❌ Invalid 2FA Code")
    }
}