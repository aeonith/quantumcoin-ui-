use actix_web::{web, HttpResponse};
use serde::Deserialize;
use crate::user::{User, load_user};
use crate::wallet::Wallet;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

pub async fn register(data: web::Json<RegisterRequest>) -> HttpResponse {
    if load_user(&data.email).is_some() {
        return HttpResponse::BadRequest().body("User already exists");
    }

    let user = User::register(&data.email, &data.password).unwrap();
    let wallet = Wallet::generate_with_password(&data.password);
    wallet.save_to_file(&user.wallet_file, &data.password);

    HttpResponse::Ok().json(serde_json::json!({ "status": "registered" }))
}

pub async fn login(data: web::Json<LoginRequest>) -> HttpResponse {
    if User::login(&data.email, &data.password) {
        HttpResponse::Ok().json(serde_json::json!({ "status": "logged_in" }))
    } else {
        HttpResponse::Unauthorized().body("Invalid login")
    }
}