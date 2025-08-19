use anyhow::{Result, Context};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderValue, StatusCode},
    response::{Html, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, error};

use crate::{
    blockchain::Blockchain,
    database::BlockchainDatabase,
    mempool::Mempool,
    p2p::{P2PNode, NetworkStats},
    rpc::AppState,
};

/// Block Explorer Server
pub struct ExplorerServer {
    addr: SocketAddr,
    app_state: AppState,
}

/// Explorer statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct ExplorerStats {
    pub total_blocks: u64,
    pub total_transactions: u64,
    pub total_addresses: u64,
    pub circulating_supply: u64,
    pub market_cap_usd: Option<f64>,
    pub avg_block_time: f64,
    pub network_hashrate: u64,
    pub difficulty: u32,
    pub mempool_size: usize,
    pub active_addresses_24h: u64,
    pub transaction_volume_24h: u64,
}

/// Recent activity for the explorer home page
#[derive(Debug, Serialize, Deserialize)]
pub struct RecentActivity {
    pub latest_blocks: Vec<BlockSummary>,
    pub latest_transactions: Vec<TransactionSummary>,
    pub network_stats: NetworkStats,
    pub chain_stats: ExplorerStats,
}

/// Block summary for listings
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockSummary {
    pub height: u64,
    pub hash: String,
    pub timestamp: i64,
    pub transaction_count: usize,
    pub size: usize,
    pub miner: String,
    pub reward: u64,
}

/// Transaction summary for listings
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionSummary {
    pub txid: String,
    pub timestamp: i64,
    pub amount: u64,
    pub fee: u64,
    pub input_count: usize,
    pub output_count: usize,
    pub confirmations: Option<u64>,
}

/// Search results
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResults {
    pub query: String,
    pub result_type: Option<String>, // "block", "transaction", "address"
    pub block: Option<BlockSummary>,
    pub transaction: Option<TransactionSummary>,
    pub address: Option<AddressSummary>,
}

/// Address summary
#[derive(Debug, Serialize, Deserialize)]
pub struct AddressSummary {
    pub address: String,
    pub balance: u64,
    pub transaction_count: usize,
    pub first_seen: Option<i64>,
    pub last_seen: Option<i64>,
}

/// Query parameters for pagination
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

/// Search query parameters
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

impl ExplorerServer {
    pub fn new(addr: SocketAddr, app_state: AppState) -> Self {
        Self {
            addr,
            app_state,
        }
    }
    
    /// Start the explorer server
    pub async fn start(&self) -> Result<()> {
        info!("Starting Block Explorer on {}", self.addr);
        
        let app = Router::new()
            // API endpoints (same as RPC server)
            .route("/api/stats", get(get_explorer_stats))
            .route("/api/recent", get(get_recent_activity))
            .route("/api/search", get(search))
            .route("/api/blocks", get(get_blocks_api))
            .route("/api/blocks/:height", get(get_block_api))
            .route("/api/transactions", get(get_transactions_api))
            .route("/api/transactions/:txid", get(get_transaction_api))
            .route("/api/addresses/:address", get(get_address_api))
            
            // Web interface
            .route("/", get(explorer_home))
            .route("/blocks", get(explorer_blocks))
            .route("/blocks/:height", get(explorer_block))
            .route("/transactions", get(explorer_transactions))
            .route("/transactions/:txid", get(explorer_transaction))
            .route("/addresses/:address", get(explorer_address))
            .route("/search", get(explorer_search))
            .route("/stats", get(explorer_stats_page))
            .route("/mempool", get(explorer_mempool))
            .route("/network", get(explorer_network))
            
            // Static assets
            .nest_service("/static", ServeDir::new("static"))
            
            .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any))
            .with_state(self.app_state.clone());
        
        let listener = tokio::net::TcpListener::bind(self.addr).await
            .context("Failed to bind explorer server")?;
            
        info!("Block Explorer listening on http://{}", self.addr);
        
        axum::serve(listener, app)
            .await
            .context("Explorer server error")?;
            
        Ok(())
    }
}

// API Endpoints

async fn get_explorer_stats(State(state): State<AppState>) -> Json<ExplorerStats> {
    let blockchain = state.blockchain.read().await;
    let mempool = state.mempool.read().await;
    
    let stats = ExplorerStats {
        total_blocks: blockchain.chain.len() as u64,
        total_transactions: blockchain.chain.iter().map(|b| b.transactions.len() as u64).sum(),
        total_addresses: 0, // TODO: Count unique addresses from database
        circulating_supply: blockchain.total_supply,
        market_cap_usd: None, // TODO: Fetch from price API
        avg_block_time: 600.0, // 10 minutes target
        network_hashrate: 1_000_000, // TODO: Calculate from difficulty
        difficulty: blockchain.difficulty,
        mempool_size: mempool.size(),
        active_addresses_24h: 0, // TODO: Calculate from recent transactions
        transaction_volume_24h: 0, // TODO: Calculate from recent blocks
    };
    
    Json(stats)
}

async fn get_recent_activity(State(state): State<AppState>) -> Json<RecentActivity> {
    let blockchain = state.blockchain.read().await;
    let mempool = state.mempool.read().await;
    let network_stats = state.p2p_node.get_stats().await;
    
    // Get latest 10 blocks
    let latest_blocks: Vec<BlockSummary> = blockchain.chain
        .iter()
        .rev()
        .take(10)
        .map(|block| BlockSummary {
            height: block.index,
            hash: block.hash.clone(),
            timestamp: block.timestamp.timestamp(),
            transaction_count: block.transactions.len(),
            size: bincode::serialize(block).map(|data| data.len()).unwrap_or(0),
            miner: "Unknown".to_string(), // TODO: Extract miner from coinbase
            reward: blockchain.get_current_mining_reward(),
        })
        .collect();
    
    // Get latest transactions from mempool
    let latest_transactions: Vec<TransactionSummary> = mempool
        .get_transactions_by_fee(10)
        .iter()
        .map(|entry| TransactionSummary {
            txid: entry.transaction.id.clone(),
            timestamp: entry.transaction.timestamp.timestamp(),
            amount: entry.transaction.outputs.iter().map(|o| o.value).sum(),
            fee: entry.transaction.calculate_fee(&std::collections::HashMap::new()).unwrap_or(0),
            input_count: entry.transaction.inputs.len(),
            output_count: entry.transaction.outputs.len(),
            confirmations: None, // Unconfirmed
        })
        .collect();
    
    let chain_stats = ExplorerStats {
        total_blocks: blockchain.chain.len() as u64,
        total_transactions: blockchain.chain.iter().map(|b| b.transactions.len() as u64).sum(),
        total_addresses: 0,
        circulating_supply: blockchain.total_supply,
        market_cap_usd: None,
        avg_block_time: 600.0,
        network_hashrate: 1_000_000,
        difficulty: blockchain.difficulty,
        mempool_size: mempool.size(),
        active_addresses_24h: 0,
        transaction_volume_24h: 0,
    };
    
    let activity = RecentActivity {
        latest_blocks,
        latest_transactions,
        network_stats,
        chain_stats,
    };
    
    Json(activity)
}

async fn search(
    Query(query): Query<SearchQuery>,
    State(state): State<AppState>,
) -> Json<SearchResults> {
    let blockchain = state.blockchain.read().await;
    let search_term = query.q.trim();
    
    let mut results = SearchResults {
        query: search_term.to_string(),
        result_type: None,
        block: None,
        transaction: None,
        address: None,
    };
    
    // Try to parse as block height
    if let Ok(height) = search_term.parse::<usize>() {
        if let Some(block) = blockchain.chain.get(height) {
            results.result_type = Some("block".to_string());
            results.block = Some(BlockSummary {
                height: block.index,
                hash: block.hash.clone(),
                timestamp: block.timestamp.timestamp(),
                transaction_count: block.transactions.len(),
                size: bincode::serialize(block).map(|data| data.len()).unwrap_or(0),
                miner: "Unknown".to_string(),
                reward: blockchain.get_current_mining_reward(),
            });
        }
    }
    
    // Try to find by block hash
    if results.result_type.is_none() {
        if let Some(block) = blockchain.chain.iter().find(|b| b.hash == search_term) {
            results.result_type = Some("block".to_string());
            results.block = Some(BlockSummary {
                height: block.index,
                hash: block.hash.clone(),
                timestamp: block.timestamp.timestamp(),
                transaction_count: block.transactions.len(),
                size: bincode::serialize(block).map(|data| data.len()).unwrap_or(0),
                miner: "Unknown".to_string(),
                reward: blockchain.get_current_mining_reward(),
            });
        }
    }
    
    // Try to find transaction by ID
    if results.result_type.is_none() {
        for block in &blockchain.chain {
            if let Some(tx) = block.transactions.iter().find(|t| t.id == search_term) {
                results.result_type = Some("transaction".to_string());
                results.transaction = Some(TransactionSummary {
                    txid: tx.id.clone(),
                    timestamp: tx.timestamp.timestamp(),
                    amount: tx.amount,
                    fee: tx.fee,
                    input_count: 1, // Simplified
                    output_count: 1, // Simplified
                    confirmations: Some(blockchain.chain.len() as u64 - block.index),
                });
                break;
            }
        }
    }
    
    // If it looks like an address
    if results.result_type.is_none() && search_term.starts_with("qtc1q") {
        results.result_type = Some("address".to_string());
        results.address = Some(AddressSummary {
            address: search_term.to_string(),
            balance: 0, // TODO: Get from database
            transaction_count: 0, // TODO: Count transactions
            first_seen: None,
            last_seen: None,
        });
    }
    
    Json(results)
}

// Web Interface Handlers

async fn explorer_home(State(state): State<AppState>) -> Html<String> {
    let recent_activity = get_recent_activity(State(state)).await.0;
    
    let html = format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>QuantumCoin Block Explorer</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; }}
        .header {{ text-align: center; margin-bottom: 40px; }}
        .stats {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin-bottom: 40px; }}
        .stat-card {{ background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .stat-value {{ font-size: 24px; font-weight: bold; color: #333; }}
        .stat-label {{ color: #666; margin-top: 5px; }}
        .section {{ background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); margin-bottom: 20px; }}
        .section h2 {{ margin-top: 0; color: #333; }}
        .block-item, .tx-item {{ padding: 10px; border-bottom: 1px solid #eee; }}
        .block-item:last-child, .tx-item:last-child {{ border-bottom: none; }}
        .hash {{ font-family: monospace; color: #666; }}
        .amount {{ font-weight: bold; color: #2e7d32; }}
        .search-box {{ width: 100%; padding: 12px; border: 2px solid #ddd; border-radius: 6px; font-size: 16px; }}
        .search-container {{ margin-bottom: 30px; }}
        .two-column {{ display: grid; grid-template-columns: 1fr 1fr; gap: 20px; }}
        @media (max-width: 768px) {{ .two-column {{ grid-template-columns: 1fr; }} }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>⚛️ QuantumCoin Explorer</h1>
            <p>Post-Quantum Cryptocurrency Block Explorer</p>
        </div>
        
        <div class="search-container">
            <input type="text" class="search-box" placeholder="Search by block height, hash, transaction ID, or address..." 
                   onkeypress="if(event.key==='Enter') search()">
        </div>
        
        <div class="stats">
            <div class="stat-card">
                <div class="stat-value">{}</div>
                <div class="stat-label">Total Blocks</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">{}</div>
                <div class="stat-label">Total Transactions</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">{:.2} QTC</div>
                <div class="stat-label">Circulating Supply</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">{}</div>
                <div class="stat-label">Connected Peers</div>
            </div>
        </div>
        
        <div class="two-column">
            <div class="section">
                <h2>Latest Blocks</h2>
                {}
            </div>
            <div class="section">
                <h2>Latest Transactions</h2>
                {}
            </div>
        </div>
    </div>
    
    <script>
        function search() {{
            const query = document.querySelector('.search-box').value;
            if (query.trim()) {{
                window.location.href = '/search?q=' + encodeURIComponent(query);
            }}
        }}
        
        // Auto-refresh every 30 seconds
        setTimeout(() => location.reload(), 30000);
    </script>
</body>
</html>
    "#,
        recent_activity.chain_stats.total_blocks,
        recent_activity.chain_stats.total_transactions,
        recent_activity.chain_stats.circulating_supply as f64 / 100_000_000.0,
        recent_activity.network_stats.connected_peers,
        recent_activity.latest_blocks.iter()
            .map(|block| format!(
                r#"<div class="block-item">
                    <div><strong>Block #{}</strong> - {} transactions</div>
                    <div class="hash">{}</div>
                    <div style="font-size: 12px; color: #888;">
                        {} • Size: {} bytes
                    </div>
                </div>"#,
                block.height,
                block.transaction_count,
                &block.hash[..16],
                format_timestamp(block.timestamp),
                block.size
            ))
            .collect::<Vec<_>>()
            .join(""),
        recent_activity.latest_transactions.iter()
            .map(|tx| format!(
                r#"<div class="tx-item">
                    <div class="hash">{}</div>
                    <div>
                        <span class="amount">{:.8} QTC</span> • 
                        Fee: {:.8} QTC
                    </div>
                    <div style="font-size: 12px; color: #888;">
                        {} • {} in → {} out
                    </div>
                </div>"#,
                &tx.txid[..16],
                tx.amount as f64 / 100_000_000.0,
                tx.fee as f64 / 100_000_000.0,
                if tx.confirmations.is_some() { "Confirmed" } else { "Unconfirmed" },
                tx.input_count,
                tx.output_count
            ))
            .collect::<Vec<_>>()
            .join("")
    );
    
    Html(html)
}

fn format_timestamp(timestamp: i64) -> String {
    use chrono::{DateTime, Utc, TimeZone};
    let dt: DateTime<Utc> = Utc.timestamp_opt(timestamp, 0).unwrap();
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

// TODO: Implement remaining web interface handlers
async fn explorer_blocks(State(_state): State<AppState>) -> Html<String> {
    Html("<h1>Blocks Page - Coming Soon</h1>".to_string())
}

async fn explorer_block(Path(_height): Path<u64>, State(_state): State<AppState>) -> Html<String> {
    Html("<h1>Block Details - Coming Soon</h1>".to_string())
}

async fn explorer_transactions(State(_state): State<AppState>) -> Html<String> {
    Html("<h1>Transactions Page - Coming Soon</h1>".to_string())
}

async fn explorer_transaction(Path(_txid): Path<String>, State(_state): State<AppState>) -> Html<String> {
    Html("<h1>Transaction Details - Coming Soon</h1>".to_string())
}

async fn explorer_address(Path(_address): Path<String>, State(_state): State<AppState>) -> Html<String> {
    Html("<h1>Address Details - Coming Soon</h1>".to_string())
}

async fn explorer_search(Query(_query): Query<SearchQuery>, State(_state): State<AppState>) -> Html<String> {
    Html("<h1>Search Results - Coming Soon</h1>".to_string())
}

async fn explorer_stats_page(State(_state): State<AppState>) -> Html<String> {
    Html("<h1>Statistics - Coming Soon</h1>".to_string())
}

async fn explorer_mempool(State(_state): State<AppState>) -> Html<String> {
    Html("<h1>Mempool - Coming Soon</h1>".to_string())
}

async fn explorer_network(State(_state): State<AppState>) -> Html<String> {
    Html("<h1>Network - Coming Soon</h1>".to_string())
}

// Placeholder API handlers
async fn get_blocks_api(State(_state): State<AppState>) -> Json<Vec<String>> {
    Json(vec![])
}

async fn get_block_api(Path(_height): Path<u64>, State(_state): State<AppState>) -> Json<Option<String>> {
    Json(None)
}

async fn get_transactions_api(State(_state): State<AppState>) -> Json<Vec<String>> {
    Json(vec![])
}

async fn get_transaction_api(Path(_txid): Path<String>, State(_state): State<AppState>) -> Json<Option<String>> {
    Json(None)
}

async fn get_address_api(Path(_address): Path<String>, State(_state): State<AppState>) -> Json<Option<String>> {
    Json(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_timestamp() {
        let timestamp = 1703116800; // 2023-12-21 00:00:00 UTC
        let formatted = format_timestamp(timestamp);
        assert!(formatted.contains("2023-12-21"));
    }
    
    #[test]
    fn test_explorer_stats_creation() {
        let stats = ExplorerStats {
            total_blocks: 1000,
            total_transactions: 5000,
            total_addresses: 100,
            circulating_supply: 1000000000000, // 10,000 QTC
            market_cap_usd: None,
            avg_block_time: 600.0,
            network_hashrate: 1_000_000,
            difficulty: 4,
            mempool_size: 50,
            active_addresses_24h: 25,
            transaction_volume_24h: 500000000000, // 5,000 QTC
        };
        
        assert_eq!(stats.total_blocks, 1000);
        assert_eq!(stats.circulating_supply, 1000000000000);
    }
}
