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

// Simplified blockchain and wallet types for backend integration
#[derive(Clone)]
pub struct AppState {
    pub users: Arc<RwLock<Vec<UserData>>>,
    pub revstop_active: Arc<RwLock<bool>>,
    pub wallet_keys: Arc<RwLock<Option<(String, String)>>>,
}

#[derive(FromForm, Serialize, Deserialize, Clone)]
struct UserData {
    username: String,
    email: String,
    password: String, // In production, this should be hashed
}

#[derive(FromForm)]
struct LoginForm {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: String,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: "Success".to_string(),
        }
    }
    
    fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            message,
        }
    }
}

// Main routes
#[get("/")]
fn index() -> &'static str {
    "Welcome to QuantumCoin Backend 🚀\n\nAvailable endpoints:\n- /register (POST)\n- /login (POST)\n- /kyc (POST)\n- /keys (GET)\n- /revstop/toggle (POST)\n- /api/* (JSON API)\n- /static/* (Static files)"
}

#[post("/register", data = "<reg_form>")]
async fn register(reg_form: Form<UserData>, state: &State<AppState>) -> Json<ApiResponse<String>> {
    let new_user = reg_form.into_inner();
    
    // Check if user already exists
    {
        let users = state.users.read().await;
        if users.iter().any(|u| u.username == new_user.username || u.email == new_user.email) {
            return Json(ApiResponse::error("User already exists".to_string()));
        }
    }
    
    // Add new user
    {
        let mut users = state.users.write().await;
        users.push(new_user.clone());
        
        // Save to file
        if let Ok(serialized) = serde_json::to_string_pretty(&*users) {
            let _ = fs::write("users.json", serialized);
        }
    }
    
    Json(ApiResponse::success(format!("User {} registered successfully", new_user.username)))
}

#[post("/login", data = "<login_form>")]
async fn login(login_form: Form<LoginForm>, state: &State<AppState>) -> Json<ApiResponse<UserData>> {
    let creds = login_form.into_inner();
    
    let users = state.users.read().await;
    for user in users.iter() {
        if user.username == creds.username && user.password == creds.password {
            let mut user_data = user.clone();
            user_data.password = "[HIDDEN]".to_string(); // Don't return password
            return Json(ApiResponse::success(user_data));
        }
    }
    
    Json(ApiResponse::error("Invalid credentials".to_string()))
}

#[post("/kyc", data = "<file>")]
async fn kyc_upload(mut file: TempFile<'_>) -> Json<ApiResponse<String>> {
    let filename = file.name().unwrap_or("kyc_file").to_string();
    let save_path = format!("uploads/{}", filename);
    
    match file.persist_to(&save_path).await {
        Ok(_) => Json(ApiResponse::success(format!("KYC file {} uploaded successfully", filename))),
        Err(_) => Json(ApiResponse::error("Upload failed".to_string())),
    }
}

#[get("/keys")]
async fn show_keys(state: &State<AppState>) -> Json<ApiResponse<Value>> {
    let keys = state.wallet_keys.read().await;
    
    if let Some((pub_key, priv_key)) = &*keys {
        Json(ApiResponse::success(json!({
            "public_key": pub_key,
            "private_key": "[HIDDEN FOR SECURITY]",
            "address": format!("QTC{}", &pub_key[..20]) // Simplified address generation
        })))
    } else {
        // Generate new keys
        let (pub_key, priv_key) = generate_quantum_keypair();
        
        let mut keys_guard = state.wallet_keys.write().await;
        *keys_guard = Some((pub_key.clone(), priv_key.clone()));
        
        Json(ApiResponse::success(json!({
            "public_key": pub_key,
            "private_key": "[HIDDEN FOR SECURITY]",
            "address": format!("QTC{}", &pub_key[..20])
        })))
    }
}

#[post("/revstop/toggle")]
async fn toggle_revstop(state: &State<AppState>) -> Json<ApiResponse<bool>> {
    let mut revstop = state.revstop_active.write().await;
    *revstop = !*revstop;
    
    Json(ApiResponse::success(*revstop))
}

// API endpoints for frontend integration
#[get("/api/blockchain/info")]
async fn api_blockchain_info() -> Json<ApiResponse<Value>> {
    // Mock blockchain info - in production, this would query the actual blockchain
    Json(ApiResponse::success(json!({
        "height": 12345,
        "difficulty": 4,
        "total_supply": 1000000000000,
        "max_supply": 22000000000000,
        "latest_block": "00001234567890abcdef...",
        "peer_count": 5,
        "mining_reward": 50000000
    })))
}

#[get("/api/wallet/balance/<address>")]
async fn api_wallet_balance(address: String) -> Json<ApiResponse<Value>> {
    // Mock balance - in production, this would query the blockchain
    let balance = 1500000000u64; // 15 QTC in satoshis
    
    Json(ApiResponse::success(json!({
        "address": address,
        "balance": balance,
        "balance_qtc": balance as f64 / 100_000_000.0
    })))
}

#[get("/api/transactions/<address>")]
async fn api_transactions(address: String) -> Json<ApiResponse<Vec<Value>>> {
    // Mock transactions - in production, this would query the blockchain
    let transactions = vec![
        json!({
            "id": "tx123...",
            "type": "receive",
            "amount": 500000000,
            "from": "QTCsender123...",
            "to": address,
            "timestamp": "2024-01-01T00:00:00Z",
            "confirmations": 6
        }),
        json!({
            "id": "tx456...",
            "type": "send",
            "amount": 250000000,
            "from": address,
            "to": "QTCrecipient456...",
            "timestamp": "2024-01-02T00:00:00Z",
            "confirmations": 3
        })
    ];
    
    Json(ApiResponse::success(transactions))
}

#[get("/api/mining/stats")]
async fn api_mining_stats() -> Json<ApiResponse<Value>> {
    Json(ApiResponse::success(json!({
        "difficulty": 4,
        "network_hashrate": 1234567890,
        "estimated_time": 600,
        "reward": 50000000,
        "active_miners": 42
    })))
}

#[post("/api/mining/start", data = "<mining_config>")]
async fn api_mining_start(mining_config: Json<Value>) -> Json<ApiResponse<String>> {
    let config = mining_config.into_inner();
    
    // In production, this would start actual mining
    Json(ApiResponse::success(format!(
        "Mining started with config: {}", 
        serde_json::to_string(&config).unwrap_or_default()
    )))
}

#[post("/api/mining/stop")]
async fn api_mining_stop() -> Json<ApiResponse<String>> {
    // In production, this would stop mining
    Json(ApiResponse::success("Mining stopped".to_string()))
}

// Utility functions
fn generate_quantum_keypair() -> (String, String) {
    // Simplified key generation - in production, use proper quantum-resistant cryptography
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    let pub_key: String = (0..64)
        .map(|_| format!("{:02x}", rng.gen::<u8>()))
        .collect();
    
    let priv_key: String = (0..64)
        .map(|_| format!("{:02x}", rng.gen::<u8>()))
        .collect();
    
    (pub_key, priv_key)
}

async fn load_users() -> Vec<UserData> {
    if let Ok(content) = fs::read_to_string("users.json") {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    }
}

#[launch]
async fn rocket() -> _ {
    // Create uploads directory if it doesn't exist
    if !Path::new("uploads").exists() {
        let _ = fs::create_dir("uploads");
    }
    
    // Initialize application state
    let app_state = AppState {
        users: Arc::new(RwLock::new(load_users().await)),
        revstop_active: Arc::new(RwLock::new(false)),
        wallet_keys: Arc::new(RwLock::new(None)),
    };
    
    rocket::build()
        .manage(app_state)
        .mount("/", routes![
            index,
            register,
            login,
            kyc_upload,
            show_keys,
            toggle_revstop,
            api_blockchain_info,
            api_wallet_balance,
            api_transactions,
            api_mining_stats,
            api_mining_start,
            api_mining_stop
        ])
        .mount("/static", FileServer::from(relative!("../static")))
        .configure(rocket::Config {
            address: "0.0.0.0".parse().unwrap(),
            port: 8080,
            ..rocket::Config::default()
        })
}
