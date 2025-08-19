use anyhow::{Result, Context};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, error, debug};

use crate::{
    blockchain::Blockchain,
    database::BlockchainDatabase,
    mempool::Mempool,
    p2p::{P2PNode, NetworkStats},
    quantum_crypto::{generate_keypair, public_key_to_address},
    transaction::SignedTransaction,
    utxo::UTXOSet,
};

/// RPC Server for QuantumCoin node
pub struct RpcServer {
    /// Server address
    addr: SocketAddr,
    
    /// Blockchain state
    blockchain: Arc<RwLock<Blockchain>>,
    
    /// Database
    database: Arc<RwLock<Option<BlockchainDatabase>>>,
    
    /// Mempool
    mempool: Arc<RwLock<Mempool>>,
    
    /// P2P node
    p2p_node: Arc<P2PNode>,
}

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub blockchain: Arc<RwLock<Blockchain>>,
    pub database: Arc<RwLock<Option<BlockchainDatabase>>>,
    pub mempool: Arc<RwLock<Mempool>>,
    pub p2p_node: Arc<P2PNode>,
}

/// API Response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: u64,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }
    
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }
}

/// Node information response
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeInfo {
    pub version: String,
    pub protocol_version: u32,
    pub chain_height: u64,
    pub best_block_hash: String,
    pub difficulty: u32,
    pub network: String,
    pub connections: usize,
    pub mempool_size: usize,
    pub total_supply: u64,
}

/// Block information response
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockInfo {
    pub height: u64,
    pub hash: String,
    pub previous_hash: String,
    pub merkle_root: String,
    pub timestamp: i64,
    pub difficulty: u32,
    pub nonce: u64,
    pub transaction_count: usize,
    pub size: usize,
    pub transactions: Vec<String>, // Transaction IDs
}

/// Transaction information response
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub txid: String,
    pub version: u32,
    pub lock_time: u32,
    pub inputs: Vec<TransactionInputInfo>,
    pub outputs: Vec<TransactionOutputInfo>,
    pub block_hash: Option<String>,
    pub block_height: Option<u64>,
    pub confirmations: Option<u64>,
    pub timestamp: i64,
    pub fee: u64,
    pub size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionInputInfo {
    pub previous_output: String,
    pub script_sig: String, // Hex encoded
    pub sequence: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionOutputInfo {
    pub value: u64,
    pub address: String,
    pub script_pubkey: String, // Hex encoded
}

/// Address information response
#[derive(Debug, Serialize, Deserialize)]
pub struct AddressInfo {
    pub address: String,
    pub balance: u64,
    pub total_received: u64,
    pub total_sent: u64,
    pub transaction_count: usize,
    pub utxo_count: usize,
}

/// Mempool information response
#[derive(Debug, Serialize, Deserialize)]
pub struct MempoolInfo {
    pub size: usize,
    pub bytes: usize,
    pub usage: usize,
    pub max_mempool: usize,
    pub mempoolmin_fee: f64,
    pub unbroadcast_count: usize,
}

/// Mining information response
#[derive(Debug, Serialize, Deserialize)]
pub struct MiningInfo {
    pub blocks: u64,
    pub current_block_size: u64,
    pub current_block_weight: u64,
    pub difficulty: f64,
    pub network_hashps: u64,
    pub pooled_tx: usize,
    pub chain: String,
    pub warnings: String,
}

/// Query parameters for blocks endpoint
#[derive(Debug, Deserialize)]
pub struct BlocksQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Query parameters for transactions endpoint
#[derive(Debug, Deserialize)]
pub struct TransactionsQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub address: Option<String>,
}

/// Send transaction request
#[derive(Debug, Deserialize)]
pub struct SendTransactionRequest {
    pub raw_transaction: String, // Hex encoded
}

impl RpcServer {
    pub fn new(
        addr: SocketAddr,
        blockchain: Arc<RwLock<Blockchain>>,
        database: Arc<RwLock<Option<BlockchainDatabase>>>,
        mempool: Arc<RwLock<Mempool>>,
        p2p_node: Arc<P2PNode>,
    ) -> Self {
        Self {
            addr,
            blockchain,
            database,
            mempool,
            p2p_node,
        }
    }
    
    /// Start the RPC server
    pub async fn start(&self) -> Result<()> {
        info!("Starting RPC server on {}", self.addr);
        
        let state = AppState {
            blockchain: Arc::clone(&self.blockchain),
            database: Arc::clone(&self.database),
            mempool: Arc::clone(&self.mempool),
            p2p_node: Arc::clone(&self.p2p_node),
        };
        
        let app = Router::new()
            // Node information
            .route("/", get(get_node_info))
            .route("/info", get(get_node_info))
            .route("/status", get(get_node_status))
            
            // Blockchain endpoints
            .route("/blocks", get(get_blocks))
            .route("/blocks/height/:height", get(get_block_by_height))
            .route("/blocks/hash/:hash", get(get_block_by_hash))
            .route("/blocks/latest", get(get_latest_block))
            
            // Transaction endpoints
            .route("/transactions", get(get_transactions))
            .route("/transactions/:txid", get(get_transaction))
            .route("/transactions/send", post(send_transaction))
            
            // Address endpoints
            .route("/addresses/:address", get(get_address_info))
            .route("/addresses/:address/balance", get(get_address_balance))
            .route("/addresses/:address/utxos", get(get_address_utxos))
            .route("/addresses/:address/transactions", get(get_address_transactions))
            
            // Mempool endpoints
            .route("/mempool", get(get_mempool_info))
            .route("/mempool/transactions", get(get_mempool_transactions))
            
            // Network endpoints
            .route("/network", get(get_network_info))
            .route("/peers", get(get_peers))
            
            // Mining endpoints
            .route("/mining", get(get_mining_info))
            
            // Utility endpoints
            .route("/utils/address/generate", post(generate_address))
            .route("/utils/fee/estimate", get(estimate_fee))
            
            // Health check
            .route("/health", get(health_check))
            
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any)
                    ),
            )
            .with_state(state);
        
        let listener = tokio::net::TcpListener::bind(self.addr).await
            .context("Failed to bind RPC server")?;
            
        info!("RPC server listening on {}", self.addr);
        
        axum::serve(listener, app)
            .await
            .context("RPC server error")?;
            
        Ok(())
    }
}

// RPC endpoint handlers

/// Get node information
async fn get_node_info(State(state): State<AppState>) -> Json<ApiResponse<NodeInfo>> {
    let blockchain = state.blockchain.read().await;
    let mempool = state.mempool.read().await;
    let network_stats = state.p2p_node.get_stats().await;
    
    let node_info = NodeInfo {
        version: "2.0.0".to_string(),
        protocol_version: crate::p2p::PROTOCOL_VERSION,
        chain_height: blockchain.chain.len() as u64,
        best_block_hash: blockchain.get_latest_block().hash.clone(),
        difficulty: blockchain.difficulty,
        network: "quantumcoin-mainnet".to_string(),
        connections: network_stats.connected_peers,
        mempool_size: mempool.size(),
        total_supply: blockchain.total_supply,
    };
    
    Json(ApiResponse::success(node_info))
}

/// Get node status (health check)
async fn get_node_status(State(state): State<AppState>) -> Json<ApiResponse<HashMap<String, serde_json::Value>>> {
    let blockchain = state.blockchain.read().await;
    let mempool = state.mempool.read().await;
    let network_stats = state.p2p_node.get_stats().await;
    
    let mut status = HashMap::new();
    status.insert("uptime".to_string(), serde_json::json!(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()));
    status.insert("chain_height".to_string(), serde_json::json!(blockchain.chain.len()));
    status.insert("mempool_size".to_string(), serde_json::json!(mempool.size()));
    status.insert("connected_peers".to_string(), serde_json::json!(network_stats.connected_peers));
    status.insert("is_syncing".to_string(), serde_json::json!(false)); // TODO: Implement sync status
    
    Json(ApiResponse::success(status))
}

/// Get blocks with pagination
async fn get_blocks(
    State(state): State<AppState>,
    Query(query): Query<BlocksQuery>,
) -> Json<ApiResponse<Vec<BlockInfo>>> {
    let blockchain = state.blockchain.read().await;
    let limit = query.limit.unwrap_or(10).min(100); // Max 100 blocks per request
    let offset = query.offset.unwrap_or(0);
    
    let total_blocks = blockchain.chain.len();
    let start = total_blocks.saturating_sub(offset + limit);
    let end = total_blocks.saturating_sub(offset);
    
    let blocks: Vec<BlockInfo> = blockchain.chain[start..end]
        .iter()
        .rev() // Show newest first
        .map(|block| BlockInfo {
            height: block.index,
            hash: block.hash.clone(),
            previous_hash: block.previous_hash.clone(),
            merkle_root: block.merkle_root.clone(),
            timestamp: block.timestamp.timestamp(),
            difficulty: block.difficulty,
            nonce: block.nonce,
            transaction_count: block.transactions.len(),
            size: bincode::serialize(block).map(|data| data.len()).unwrap_or(0),
            transactions: block.transactions.iter().map(|tx| tx.id.clone()).collect(),
        })
        .collect();
    
    Json(ApiResponse::success(blocks))
}

/// Get block by height
async fn get_block_by_height(
    Path(height): Path<u64>,
    State(state): State<AppState>,
) -> Json<ApiResponse<BlockInfo>> {
    let blockchain = state.blockchain.read().await;
    
    if let Some(block) = blockchain.chain.get(height as usize) {
        let block_info = BlockInfo {
            height: block.index,
            hash: block.hash.clone(),
            previous_hash: block.previous_hash.clone(),
            merkle_root: block.merkle_root.clone(),
            timestamp: block.timestamp.timestamp(),
            difficulty: block.difficulty,
            nonce: block.nonce,
            transaction_count: block.transactions.len(),
            size: bincode::serialize(block).map(|data| data.len()).unwrap_or(0),
            transactions: block.transactions.iter().map(|tx| tx.id.clone()).collect(),
        };
        Json(ApiResponse::success(block_info))
    } else {
        Json(ApiResponse::error("Block not found".to_string()))
    }
}

/// Get block by hash
async fn get_block_by_hash(
    Path(hash): Path<String>,
    State(state): State<AppState>,
) -> Json<ApiResponse<BlockInfo>> {
    let blockchain = state.blockchain.read().await;
    
    if let Some(block) = blockchain.chain.iter().find(|b| b.hash == hash) {
        let block_info = BlockInfo {
            height: block.index,
            hash: block.hash.clone(),
            previous_hash: block.previous_hash.clone(),
            merkle_root: block.merkle_root.clone(),
            timestamp: block.timestamp.timestamp(),
            difficulty: block.difficulty,
            nonce: block.nonce,
            transaction_count: block.transactions.len(),
            size: bincode::serialize(block).map(|data| data.len()).unwrap_or(0),
            transactions: block.transactions.iter().map(|tx| tx.id.clone()).collect(),
        };
        Json(ApiResponse::success(block_info))
    } else {
        Json(ApiResponse::error("Block not found".to_string()))
    }
}

/// Get latest block
async fn get_latest_block(State(state): State<AppState>) -> Json<ApiResponse<BlockInfo>> {
    let blockchain = state.blockchain.read().await;
    let block = blockchain.get_latest_block();
    
    let block_info = BlockInfo {
        height: block.index,
        hash: block.hash.clone(),
        previous_hash: block.previous_hash.clone(),
        merkle_root: block.merkle_root.clone(),
        timestamp: block.timestamp.timestamp(),
        difficulty: block.difficulty,
        nonce: block.nonce,
        transaction_count: block.transactions.len(),
        size: bincode::serialize(block).map(|data| data.len()).unwrap_or(0),
        transactions: block.transactions.iter().map(|tx| tx.id.clone()).collect(),
    };
    
    Json(ApiResponse::success(block_info))
}

/// Get mempool information
async fn get_mempool_info(State(state): State<AppState>) -> Json<ApiResponse<MempoolInfo>> {
    let mempool = state.mempool.read().await;
    let stats = mempool.get_mempool_stats();
    
    let mempool_info = MempoolInfo {
        size: stats.transaction_count,
        bytes: 0, // TODO: Calculate total bytes
        usage: stats.transaction_count,
        max_mempool: 10000, // TODO: Get from config
        mempoolmin_fee: stats.min_fee_per_byte,
        unbroadcast_count: 0, // TODO: Track unbroadcast transactions
    };
    
    Json(ApiResponse::success(mempool_info))
}

/// Get address balance
async fn get_address_balance(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> Json<ApiResponse<u64>> {
    let database = state.database.read().await;
    
    if let Some(db) = database.as_ref() {
        match db.get_balance(&address).await {
            Ok(balance) => Json(ApiResponse::success(balance)),
            Err(e) => Json(ApiResponse::error(format!("Database error: {}", e))),
        }
    } else {
        Json(ApiResponse::error("Database not available".to_string()))
    }
}

/// Get network information
async fn get_network_info(State(state): State<AppState>) -> Json<ApiResponse<NetworkStats>> {
    let stats = state.p2p_node.get_stats().await;
    Json(ApiResponse::success(stats))
}

/// Generate new address
async fn generate_address() -> Json<ApiResponse<HashMap<String, String>>> {
    let (public_key, private_key) = generate_keypair();
    let address = public_key_to_address(&public_key);
    
    let mut response = HashMap::new();
    response.insert("address".to_string(), address);
    response.insert("public_key".to_string(), public_key);
    response.insert("private_key".to_string(), private_key);
    
    Json(ApiResponse::success(response))
}

/// Health check endpoint
async fn health_check() -> Json<ApiResponse<HashMap<String, String>>> {
    let mut health = HashMap::new();
    health.insert("status".to_string(), "healthy".to_string());
    health.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());
    
    Json(ApiResponse::success(health))
}

// TODO: Implement remaining endpoints
async fn get_transactions(
    State(_state): State<AppState>,
    Query(_query): Query<TransactionsQuery>,
) -> Json<ApiResponse<Vec<String>>> {
    Json(ApiResponse::error("Not implemented yet".to_string()))
}

async fn get_transaction(
    Path(_txid): Path<String>,
    State(_state): State<AppState>,
) -> Json<ApiResponse<String>> {
    Json(ApiResponse::error("Not implemented yet".to_string()))
}

async fn send_transaction(
    State(_state): State<AppState>,
    Json(_request): Json<SendTransactionRequest>,
) -> Json<ApiResponse<String>> {
    Json(ApiResponse::error("Not implemented yet".to_string()))
}

async fn get_address_info(
    Path(_address): Path<String>,
    State(_state): State<AppState>,
) -> Json<ApiResponse<AddressInfo>> {
    Json(ApiResponse::error("Not implemented yet".to_string()))
}

async fn get_address_utxos(
    Path(_address): Path<String>,
    State(_state): State<AppState>,
) -> Json<ApiResponse<Vec<String>>> {
    Json(ApiResponse::error("Not implemented yet".to_string()))
}

async fn get_address_transactions(
    Path(_address): Path<String>,
    State(_state): State<AppState>,
) -> Json<ApiResponse<Vec<String>>> {
    Json(ApiResponse::error("Not implemented yet".to_string()))
}

async fn get_mempool_transactions(State(_state): State<AppState>) -> Json<ApiResponse<Vec<String>>> {
    Json(ApiResponse::error("Not implemented yet".to_string()))
}

async fn get_peers(State(_state): State<AppState>) -> Json<ApiResponse<Vec<String>>> {
    Json(ApiResponse::error("Not implemented yet".to_string()))
}

async fn get_mining_info(State(_state): State<AppState>) -> Json<ApiResponse<MiningInfo>> {
    Json(ApiResponse::error("Not implemented yet".to_string()))
}

async fn estimate_fee(State(_state): State<AppState>) -> Json<ApiResponse<f64>> {
    Json(ApiResponse::error("Not implemented yet".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    
    #[tokio::test]
    async fn test_health_check() {
        let app = Router::new()
            .route("/health", get(health_check));
        
        let server = TestServer::new(app).unwrap();
        let response = server.get("/health").await;
        
        assert_eq!(response.status_code(), 200);
        
        let body: ApiResponse<HashMap<String, String>> = response.json();
        assert!(body.success);
        assert!(body.data.is_some());
    }
    
    #[tokio::test]
    async fn test_generate_address() {
        let app = Router::new()
            .route("/utils/address/generate", post(generate_address));
        
        let server = TestServer::new(app).unwrap();
        let response = server.post("/utils/address/generate").await;
        
        assert_eq!(response.status_code(), 200);
        
        let body: ApiResponse<HashMap<String, String>> = response.json();
        assert!(body.success);
        assert!(body.data.is_some());
        
        let data = body.data.unwrap();
        assert!(data.contains_key("address"));
        assert!(data.contains_key("public_key"));
        assert!(data.contains_key("private_key"));
    }
}
