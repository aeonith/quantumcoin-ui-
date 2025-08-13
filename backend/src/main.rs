#[macro_use]
extern crate rocket;

use rocket::fs::{FileServer, relative, TempFile};
use rocket::form::Form;
use rocket::response::Redirect;
use rocket::serde::{Serialize, Deserialize, json::Json};
use rocket::{State, get, post, routes, launch, Build, Rocket};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::{Value, json};

mod wallet;
mod revstop;
mod blockchain;
mod rpc;

use crate::blockchain::{Blockchain, Transaction};
use crate::rpc::{RpcServer, RpcRequest, RpcResponse};

#[derive(FromForm, Serialize, Deserialize)]
struct UserData {
    username: String,
    email: String,
    password: String,
}

#[post("/register", data = "<reg_form>")]
fn register(reg_form: Form<UserData>) -> Redirect {
    let new_user = reg_form.into_inner();

    let mut users: Vec<UserData> = if let Ok(content) = fs::read_to_string("users.json") {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    users.push(new_user);
    let serialized = serde_json::to_string_pretty(&users).unwrap();
    fs::write("users.json", serialized).expect("Failed to write users.json");

    Redirect::to("/static/index.html")
}

#[derive(FromForm)]
struct LoginForm {
    username: String,
    password: String,
}

#[post("/login", data = "<login_form>")]
fn login(login_form: Form<LoginForm>) -> String {
    let creds = login_form.into_inner();

    if let Ok(content) = fs::read_to_string("users.json") {
        let users: Vec<UserData> = serde_json::from_str(&content).unwrap_or_default();
        for user in users {
            if user.username == creds.username && user.password == creds.password {
                return format!("Login successful. Welcome, {}!", user.username);
            }
        }
    }

    "Invalid credentials.".to_string()
}

#[post("/kyc", data = "<file>")]
async fn kyc_upload(mut file: TempFile<'_>) -> &'static str {
    let save_path = format!("uploads/{}", file.name().unwrap_or("kyc_file"));
    if let Err(_) = file.persist_to(save_path).await {
        return "Upload failed.";
    }
    "KYC upload successful."
}

#[get("/keys")]
fn show_keys() -> String {
    let (pub_key, priv_key) = wallet::get_keys();
    format!("Public Key:\n{}\n\nPrivate Key:\n{}", pub_key, priv_key)
}

#[post("/revstop/toggle")]
fn toggle_revstop() -> String {
    if revstop::is_revstop_active() {
        revstop::deactivate();
        "RevStop Deactivated.".to_string()
    } else {
        revstop::activate();
        "RevStop Activated.".to_string()
    }
}

#[get("/")]
fn index() -> &'static str {
    "Welcome to QuantumCoin Backend 🚀"
}

#[get("/blockchain")]
fn get_blockchain(blockchain_state: &State<Arc<RwLock<Blockchain>>>) -> Json<Vec<blockchain::Block>> {
    let blockchain = futures::executor::block_on(blockchain_state.read());
    Json(blockchain.chain.clone())
}

#[get("/balance/<address>")]
fn get_balance(address: String, blockchain_state: &State<Arc<RwLock<Blockchain>>>) -> Json<Value> {
    let blockchain = futures::executor::block_on(blockchain_state.read());
    let balance = blockchain.get_balance(&address);
    Json(json!({"address": address, "balance": balance}))
}

#[post("/transaction", data = "<transaction>")]
fn create_transaction(
    transaction: Json<Transaction>,
    blockchain_state: &State<Arc<RwLock<Blockchain>>>
) -> Json<Value> {
    let mut blockchain = futures::executor::block_on(blockchain_state.write());
    blockchain.create_transaction(transaction.into_inner());
    Json(json!({"status": "Transaction added to pending pool"}))
}

#[post("/mine/<reward_address>")]
fn mine_block(
    reward_address: String,
    blockchain_state: &State<Arc<RwLock<Blockchain>>>
) -> Json<blockchain::Block> {
    let mut blockchain = futures::executor::block_on(blockchain_state.write());
    let mined_block = blockchain.mine_pending_transactions(reward_address);
    Json(mined_block)
}

#[launch]
fn rocket() -> _ {
    if !Path::new("uploads").exists() {
        fs::create_dir("uploads").unwrap();
    }

    let blockchain = Arc::new(RwLock::new(Blockchain::new()));
    
    // Start RPC server in background
    let rpc_blockchain = Arc::clone(&blockchain);
    tokio::spawn(async move {
        let rpc_server = RpcServer::new(rpc_blockchain);
        if let Err(e) = rpc_server.start("127.0.0.1:18332").await {
            println!("RPC server error: {}", e);
        }
    });

    rocket::build()
        .manage(blockchain)
        .mount("/", routes![
            index, register, login, kyc_upload, show_keys, toggle_revstop,
            get_blockchain, get_balance, create_transaction, mine_block
        ])
        .mount("/static", FileServer::from(relative!("static")))
        .configure(rocket::Config {
            address: "0.0.0.0".parse().unwrap(),
            port: 8080,
            ..rocket::Config::default()
        })
}