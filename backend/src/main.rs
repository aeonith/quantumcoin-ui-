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
mod ai_integration;

use crate::blockchain::{Blockchain, Transaction};
use crate::rpc::{RpcServer, RpcRequest, RpcResponse};
use crate::ai_integration::{AIState, SentinelOutput};

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
    Json(json!({
        "address": address, 
        "balance": balance,
        "confirmed_balance": balance,
        "pending_balance": 0,
        "last_updated": chrono::Utc::now().to_rfc3339()
    }))
}

#[post("/wallet/generate")]
fn generate_wallet() -> Json<Value> {
    // Generate REAL quantum-resistant wallet
    let (private_key, public_key, address) = wallet::generate_quantum_keypair();
    Json(json!({
        "success": true,
        "address": address,
        "public_key": public_key,
        "created_at": chrono::Utc::now().to_rfc3339(),
        "quantum_resistant": true
    }))
}

#[get("/network/stats")]
fn get_network_stats(blockchain_state: &State<Arc<RwLock<Blockchain>>>) -> Json<Value> {
    let blockchain = futures::executor::block_on(blockchain_state.read());
    Json(json!({
        "height": blockchain.get_height(),
        "difficulty": blockchain.get_difficulty(),
        "hash_rate": blockchain.get_network_hashrate(),
        "total_supply": blockchain.get_total_supply(),
        "circulating_supply": blockchain.get_circulating_supply(),
        "active_nodes": blockchain.get_active_nodes(),
        "mempool_size": blockchain.get_mempool_size(),
        "last_block_time": blockchain.get_last_block_time(),
        "network_version": "1.0.0"
    }))
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

#[derive(Deserialize)]
struct CreditRequest {
    address: String,
    amount: f64,
    reason: Option<String>,
}

#[post("/wallet/credit", data = "<credit_req>")]
fn credit_wallet(
    credit_req: Json<CreditRequest>,
    blockchain_state: &State<Arc<RwLock<Blockchain>>>
) -> Json<Value> {
    let mut blockchain = futures::executor::block_on(blockchain_state.write());
    
    // Create a credit transaction (from exchange)
    let transaction = blockchain::Transaction {
        id: format!("exchange_{}", chrono::Utc::now().timestamp()),
        from: "EXCHANGE".to_string(),
        to: credit_req.address.clone(),
        amount: credit_req.amount,
        timestamp: chrono::Utc::now(),
        signature: "EXCHANGE_VERIFIED".to_string(),
    };
    
    blockchain.create_transaction(transaction);
    let new_balance = blockchain.get_balance(&credit_req.address);
    
    Json(json!({
        "success": true,
        "amount": credit_req.amount,
        "newBalance": new_balance,
        "reason": credit_req.reason.as_ref().unwrap_or(&"CREDIT".to_string()),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

#[launch]
fn rocket() -> _ {
    if !Path::new("uploads").exists() {
        fs::create_dir("uploads").unwrap();
    }

    let blockchain = Arc::new(RwLock::new(Blockchain::new()));
    let ai_state = Arc::new(RwLock::new(AIState::default()));
    
    // Start RPC server in background
    let rpc_blockchain = Arc::clone(&blockchain);
    tokio::spawn(async move {
        let rpc_server = RpcServer::new(rpc_blockchain);
        if let Err(e) = rpc_server.start("127.0.0.1:18332").await {
            println!("RPC server error: {}", e);
        }
    });

    // Start AI Sentinel in background
    let ai_blockchain = Arc::clone(&blockchain);
    let ai_state_clone = Arc::clone(&ai_state);
    tokio::spawn(async move {
        loop {
            // AI continuously monitors and optimizes blockchain
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            
            let blockchain_read = ai_blockchain.read().await;
            if let Some(latest_block) = blockchain_read.chain.last() {
                tracing::info!("🤖 AI Sentinel analyzing block #{}", latest_block.index);
                // In full implementation, this would trigger AI analysis
            }
        }
    });

    rocket::build()
        .manage(blockchain)
        .manage(ai_state)
        .mount("/", routes![
            index, register, login, kyc_upload, show_keys, toggle_revstop,
            get_blockchain, get_balance, create_transaction, mine_block, credit_wallet,
            ai_integration::update_ai_optimizations, ai_integration::get_ai_status,
            ai_integration::get_network_metrics, ai_integration::get_latest_block
        ])
        .mount("/static", FileServer::from(relative!("static")))
        .configure(rocket::Config {
            address: "0.0.0.0".parse().unwrap(),
            port: 8080,
            ..rocket::Config::default()
        })
}