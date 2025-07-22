use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::user::{User, hash_password, verify_password, generate_2fa_secret, verify_2fa, load_users, save_users};
use crate::wallet::Wallet;

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct SignupResponse {
    pub message: String,
    pub secret_2fa_url: String,
    pub wallet_address: String,
}

#[post("/signup")]
pub async fn signup(req: web::Json<SignupRequest>, wallet: web::Data<Arc<Mutex<Wallet>>>) -> impl Responder {
    let mut users = load_users();

    if users.contains_key(&req.email) {
        return HttpResponse::BadRequest().body("User already exists");
    }

    let wallet_address = {
        let w = wallet.lock().unwrap();
        w.get_address()
    };

    let secret_2fa = generate_2fa_secret(&req.email);

    let user = User {
        email: req.email.clone(),
        password_hash: hash_password(&req.password),
        wallet_address,
        two_fa_secret: secret_2fa.clone(),
    };

    users.insert(req.email.clone(), user);
    save_users(&users);

    HttpResponse::Ok().json(SignupResponse {
        message: "Signup successful. Scan your 2FA QR or save the secret.".to_string(),
        secret_2fa_url: secret_2fa,
        wallet_address: users[&req.email].wallet_address.clone(),
    })
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub token: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub message: String,
    pub wallet_address: String,
}

#[post("/login")]
pub async fn login(req: web::Json<LoginRequest>) -> impl Responder {
    let users = load_users();

    if let Some(user) = users.get(&req.email) {
        if verify_password(&req.password, &user.password_hash) && verify_2fa(&user.two_fa_secret, &req.token) {
            return HttpResponse::Ok().json(LoginResponse {
                message: "Login successful".to_string(),
                wallet_address: user.wallet_address.clone(),
            });
        }
    }

    HttpResponse::Unauthorized().body("Invalid credentials or 2FA token")
}