use crate::blockchain::Blockchain;
use crate::block::Block;
use crate::transaction::Transaction;
use crate::wallet::Wallet;
use crate::network::NetworkManager;
use serde::{Serialize, Deserialize};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::RwLock;
use rocket::serde::{Serialize as RocketSerialize, Deserialize as RocketDeserialize};
use rocket::{State, get, post, routes, launch, Build, Rocket};
use rocket::serde::json::Json;
use anyhow::Result;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
    pub id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub result: Option<Value>,
    pub error: Option<RpcError>,
    pub id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Clone)]
pub struct RpcServer {
    pub blockchain: Arc<RwLock<Blockchain>>,
    pub network: Option<Arc<NetworkManager>>,
    pub wallet: Arc<RwLock<Wallet>>,
}

impl RpcServer {
    pub fn new(blockchain: Arc<RwLock<Blockchain>>, wallet: Arc<RwLock<Wallet>>) -> Self {
        Self {
            blockchain,
            network: None,
            wallet,
        }
    }

    pub fn with_network(mut self, network: Arc<NetworkManager>) -> Self {
        self.network = Some(network);
        self
    }

    pub async fn handle_request(&self, request: RpcRequest) -> RpcResponse {
        let result = match request.method.as_str() {
            // Blockchain info methods
            "getblockchaininfo" => self.get_blockchain_info().await,
            "getblockcount" => self.get_block_count().await,
            "getbestblockhash" => self.get_best_block_hash().await,
            "getdifficulty" => self.get_difficulty().await,
            "gettotalsupply" => self.get_total_supply().await,
            
            // Block methods
            "getblock" => self.get_block(&request.params).await,
            "getblockheader" => self.get_block_header(&request.params).await,
            "getblockhash" => self.get_block_hash(&request.params).await,
            
            // Transaction methods
            "gettransaction" => self.get_transaction(&request.params).await,
            "sendrawtransaction" => self.send_raw_transaction(&request.params).await,
            "getmempool" => self.get_mempool().await,
            "getmempoolinfo" => self.get_mempool_info().await,
            
            // Wallet methods
            "getbalance" => self.get_balance(&request.params).await,
            "getnewaddress" => self.get_new_address().await,
            "sendtoaddress" => self.send_to_address(&request.params).await,
            "listtransactions" => self.list_transactions(&request.params).await,
            "getaddressinfo" => self.get_address_info(&request.params).await,
            
            // Mining methods
            "getmininginfo" => self.get_mining_info().await,
            "submitblock" => self.submit_block(&request.params).await,
            "getblocktemplate" => self.get_block_template().await,
            
            // Network methods
            "getpeerinfo" => self.get_peer_info().await,
            "getnetworkinfo" => self.get_network_info().await,
            "addnode" => self.add_node(&request.params).await,
            
            // Utility methods
            "validateaddress" => self.validate_address(&request.params).await,
            "estimatefee" => self.estimate_fee(&request.params).await,
            
            _ => Err(RpcError {
                code: -32601,
                message: "Method not found".to_string(),
                data: None,
            }),
        };

        match result {
            Ok(result) => RpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(result),
                error: None,
                id: request.id,
            },
            Err(error) => RpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(error),
                id: request.id,
            },
        }
    }

    // Blockchain info methods
    async fn get_blockchain_info(&self) -> Result<Value, RpcError> {
        let blockchain = self.blockchain.read().await;
        Ok(json!({
            "chain": "QuantumCoin",
            "blocks": blockchain.get_chain_height(),
            "difficulty": blockchain.get_difficulty(),
            "bestblockhash": blockchain.get_latest_block_hash(),
            "totalsupply": blockchain.get_total_supply(),
            "maxsupply": blockchain.get_max_supply(),
            "circulationpercentage": blockchain.get_circulation_percentage(),
            "halving_interval": blockchain.halving_interval,
            "quantum_resistant": true
        }))
    }

    async fn get_block_count(&self) -> Result<Value, RpcError> {
        let blockchain = self.blockchain.read().await;
        Ok(json!(blockchain.get_chain_height()))
    }

    async fn get_best_block_hash(&self) -> Result<Value, RpcError> {
        let blockchain = self.blockchain.read().await;
        Ok(json!(blockchain.get_latest_block_hash()))
    }

    async fn get_difficulty(&self) -> Result<Value, RpcError> {
        let blockchain = self.blockchain.read().await;
        Ok(json!(blockchain.get_difficulty()))
    }

    async fn get_total_supply(&self) -> Result<Value, RpcError> {
        let blockchain = self.blockchain.read().await;
        Ok(json!(blockchain.get_total_supply()))
    }

    // Block methods
    async fn get_block(&self, params: &Value) -> Result<Value, RpcError> {
        let hash = params.get(0)
            .and_then(|v| v.as_str())
            .ok_or_else(|| RpcError {
                code: -1,
                message: "Block hash required".to_string(),
                data: None,
            })?;

        let blockchain = self.blockchain.read().await;
        if let Some(block) = blockchain.get_block_by_hash(hash) {
            Ok(json!({
                "hash": block.hash,
                "height": block.index,
                "timestamp": block.timestamp,
                "previous_hash": block.previous_hash,
                "nonce": block.nonce,
                "transactions": block.transactions,
                "transaction_count": block.transactions.len(),
                "size": serde_json::to_string(block).unwrap_or_default().len()
            }))
        } else {
            Err(RpcError {
                code: -5,
                message: "Block not found".to_string(),
                data: None,
            })
        }
    }

    async fn get_block_header(&self, params: &Value) -> Result<Value, RpcError> {
        let hash = params.get(0)
            .and_then(|v| v.as_str())
            .ok_or_else(|| RpcError {
                code: -1,
                message: "Block hash required".to_string(),
                data: None,
            })?;

        let blockchain = self.blockchain.read().await;
        if let Some(block) = blockchain.get_block_by_hash(hash) {
            Ok(json!({
                "hash": block.hash,
                "height": block.index,
                "timestamp": block.timestamp,
                "previous_hash": block.previous_hash,
                "nonce": block.nonce,
                "transaction_count": block.transactions.len()
            }))
        } else {
            Err(RpcError {
                code: -5,
                message: "Block not found".to_string(),
                data: None,
            })
        }
    }

    async fn get_block_hash(&self, params: &Value) -> Result<Value, RpcError> {
        let height = params.get(0)
            .and_then(|v| v.as_u64())
            .ok_or_else(|| RpcError {
                code: -1,
                message: "Block height required".to_string(),
                data: None,
            })?;

        let blockchain = self.blockchain.read().await;
        if let Some(block) = blockchain.get_block_by_index(height) {
            Ok(json!(block.hash))
        } else {
            Err(RpcError {
                code: -5,
                message: "Block not found".to_string(),
                data: None,
            })
        }
    }

    // Transaction methods
    async fn get_transaction(&self, params: &Value) -> Result<Value, RpcError> {
        let tx_id = params.get(0)
            .and_then(|v| v.as_str())
            .ok_or_else(|| RpcError {
                code: -1,
                message: "Transaction ID required".to_string(),
                data: None,
            })?;

        let blockchain = self.blockchain.read().await;
        
        // Search in all blocks for the transaction
        for block in &blockchain.chain {
            for tx in &block.transactions {
                if tx.id == tx_id {
                    return Ok(json!({
                        "txid": tx.id,
                        "sender": tx.sender,
                        "recipient": tx.recipient,
                        "amount": tx.amount,
                        "fee": tx.fee,
                        "timestamp": tx.timestamp,
                        "signature": tx.signature,
                        "confirmations": blockchain.chain.len() - block.index as usize,
                        "blockhash": block.hash,
                        "blockheight": block.index
                    }));
                }
            }
        }

        Err(RpcError {
            code: -5,
            message: "Transaction not found".to_string(),
            data: None,
        })
    }

    async fn send_raw_transaction(&self, params: &Value) -> Result<Value, RpcError> {
        let tx_data = params.get(0)
            .and_then(|v| v.as_str())
            .ok_or_else(|| RpcError {
                code: -1,
                message: "Raw transaction data required".to_string(),
                data: None,
            })?;

        let transaction: Transaction = serde_json::from_str(tx_data)
            .map_err(|_| RpcError {
                code: -22,
                message: "Invalid transaction format".to_string(),
                data: None,
            })?;

        let mut blockchain = self.blockchain.write().await;
        blockchain.add_transaction(transaction.clone())
            .map_err(|e| RpcError {
                code: -26,
                message: format!("Transaction rejected: {}", e),
                data: None,
            })?;

        // Broadcast to network if available
        if let Some(network) = &self.network {
            network.broadcast_transaction(transaction.clone()).await;
        }

        Ok(json!(transaction.id))
    }

    async fn get_mempool(&self) -> Result<Value, RpcError> {
        let blockchain = self.blockchain.read().await;
        let transactions = blockchain.get_pending_transactions();
        Ok(json!(transactions))
    }

    async fn get_mempool_info(&self) -> Result<Value, RpcError> {
        let blockchain = self.blockchain.read().await;
        Ok(json!({
            "size": blockchain.get_pending_transaction_count(),
            "bytes": blockchain.get_pending_transaction_count() * 500, // Rough estimate
            "mempoolminfee": 1000
        }))
    }

    // Wallet methods
    async fn get_balance(&self, params: &Value) -> Result<Value, RpcError> {
        let address = params.get(0)
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| {
                // Default to wallet's primary address
                &self.wallet.blocking_read().address
            });

        let blockchain = self.blockchain.read().await;
        let balance = blockchain.get_balance(address);
        Ok(json!(balance))
    }

    async fn get_new_address(&self) -> Result<Value, RpcError> {
        let wallet = self.wallet.read().await;
        Ok(json!(wallet.address.clone()))
    }

    async fn send_to_address(&self, params: &Value) -> Result<Value, RpcError> {
        let address = params.get(0)
            .and_then(|v| v.as_str())
            .ok_or_else(|| RpcError {
                code: -1,
                message: "Recipient address required".to_string(),
                data: None,
            })?;

        let amount = params.get(1)
            .and_then(|v| v.as_u64())
            .ok_or_else(|| RpcError {
                code: -1,
                message: "Amount required".to_string(),
                data: None,
            })?;

        let wallet = self.wallet.read().await;
        let transaction = wallet.create_transaction(address.to_string(), amount, 1000)
            .map_err(|e| RpcError {
                code: -4,
                message: format!("Transaction creation failed: {}", e),
                data: None,
            })?;

        let mut blockchain = self.blockchain.write().await;
        blockchain.add_transaction(transaction.clone())
            .map_err(|e| RpcError {
                code: -26,
                message: format!("Transaction rejected: {}", e),
                data: None,
            })?;

        // Broadcast to network if available
        drop(blockchain);
        if let Some(network) = &self.network {
            network.broadcast_transaction(transaction.clone()).await;
        }

        Ok(json!(transaction.id))
    }

    async fn list_transactions(&self, _params: &Value) -> Result<Value, RpcError> {
        let wallet = self.wallet.read().await;
        let blockchain = self.blockchain.read().await;
        
        let mut transactions = Vec::new();
        
        // Get transactions involving this wallet
        for block in &blockchain.chain {
            for tx in &block.transactions {
                if tx.sender == wallet.address || tx.recipient == wallet.address {
                    transactions.push(json!({
                        "txid": tx.id,
                        "sender": tx.sender,
                        "recipient": tx.recipient,
                        "amount": tx.amount,
                        "fee": tx.fee,
                        "timestamp": tx.timestamp,
                        "confirmations": blockchain.chain.len() - block.index as usize,
                        "category": if tx.recipient == wallet.address { "receive" } else { "send" }
                    }));
                }
            }
        }

        Ok(json!(transactions))
    }

    async fn get_address_info(&self, params: &Value) -> Result<Value, RpcError> {
        let address = params.get(0)
            .and_then(|v| v.as_str())
            .ok_or_else(|| RpcError {
                code: -1,
                message: "Address required".to_string(),
                data: None,
            })?;

        let blockchain = self.blockchain.read().await;
        let balance = blockchain.get_balance(address);
        
        Ok(json!({
            "address": address,
            "balance": balance,
            "ismine": address == self.wallet.read().await.address,
            "isvalid": true // Simplified validation
        }))
    }

    // Mining methods
    async fn get_mining_info(&self) -> Result<Value, RpcError> {
        let blockchain = self.blockchain.read().await;
        Ok(json!({
            "blocks": blockchain.get_chain_height(),
            "difficulty": blockchain.get_difficulty(),
            "networkhashps": blockchain.estimate_network_hashrate(),
            "pooledtx": blockchain.get_pending_transaction_count(),
            "chain": "QuantumCoin",
            "warnings": ""
        }))
    }

    async fn submit_block(&self, params: &Value) -> Result<Value, RpcError> {
        let block_data = params.get(0)
            .and_then(|v| v.as_str())
            .ok_or_else(|| RpcError {
                code: -1,
                message: "Block data required".to_string(),
                data: None,
            })?;

        let block: Block = serde_json::from_str(block_data)
            .map_err(|_| RpcError {
                code: -22,
                message: "Invalid block format".to_string(),
                data: None,
            })?;

        let mut blockchain = self.blockchain.write().await;
        blockchain.add_block(block.clone())
            .map_err(|e| RpcError {
                code: -25,
                message: format!("Block rejected: {}", e),
                data: None,
            })?;

        // Broadcast to network if available
        drop(blockchain);
        if let Some(network) = &self.network {
            network.broadcast_block(block).await;
        }

        Ok(json!(null))
    }

    async fn get_block_template(&self) -> Result<Value, RpcError> {
        let blockchain = self.blockchain.read().await;
        let wallet = self.wallet.read().await;
        
        // Create a block template for mining
        drop(blockchain);
        let mut blockchain = self.blockchain.write().await;
        let block = blockchain.create_block(&wallet.address)
            .map_err(|e| RpcError {
                code: -1,
                message: format!("Failed to create block template: {}", e),
                data: None,
            })?;

        Ok(json!({
            "version": 1,
            "previousblockhash": block.previous_hash,
            "transactions": block.transactions,
            "coinbasevalue": blockchain.get_current_mining_reward(),
            "target": "0".repeat(blockchain.difficulty),
            "mintime": block.timestamp,
            "mutable": ["time", "transactions", "prevblock"],
            "noncerange": "00000000ffffffff",
            "sigoplimit": 20000,
            "sizelimit": 1000000
        }))
    }

    // Network methods
    async fn get_peer_info(&self) -> Result<Value, RpcError> {
        if let Some(network) = &self.network {
            let peers = network.get_peers().await;
            let peer_info: Vec<Value> = peers.into_iter().map(|peer| {
                json!({
                    "id": peer.node_id,
                    "addr": peer.address.to_string(),
                    "version": peer.version,
                    "connected": peer.connected,
                    "lastseen": peer.last_seen.elapsed().as_secs()
                })
            }).collect();
            Ok(json!(peer_info))
        } else {
            Ok(json!([]))
        }
    }

    async fn get_network_info(&self) -> Result<Value, RpcError> {
        let connections = if let Some(network) = &self.network {
            network.get_peer_count().await
        } else {
            0
        };

        Ok(json!({
            "version": 10000,
            "subversion": "/QuantumCoin:1.0.0/",
            "protocolversion": 1,
            "localservices": "0000000000000001",
            "connections": connections,
            "networks": [{
                "name": "ipv4",
                "limited": false,
                "reachable": true,
                "proxy": "",
                "proxy_randomize_credentials": false
            }],
            "relayfee": 0.00001000,
            "incrementalfee": 0.00001000,
            "localaddresses": [],
            "warnings": ""
        }))
    }

    async fn add_node(&self, params: &Value) -> Result<Value, RpcError> {
        let node_addr = params.get(0)
            .and_then(|v| v.as_str())
            .ok_or_else(|| RpcError {
                code: -1,
                message: "Node address required".to_string(),
                data: None,
            })?;

        if let Some(network) = &self.network {
            let addr = node_addr.parse()
                .map_err(|_| RpcError {
                    code: -1,
                    message: "Invalid address format".to_string(),
                    data: None,
                })?;
            
            network.add_bootstrap_node(addr).await;
            Ok(json!(null))
        } else {
            Err(RpcError {
                code: -1,
                message: "Network not available".to_string(),
                data: None,
            })
        }
    }

    // Utility methods
    async fn validate_address(&self, params: &Value) -> Result<Value, RpcError> {
        let address = params.get(0)
            .and_then(|v| v.as_str())
            .ok_or_else(|| RpcError {
                code: -1,
                message: "Address required".to_string(),
                data: None,
            })?;

        // Simplified validation - in production, you'd check format, checksum, etc.
        let is_valid = address.len() > 20 && address.chars().all(|c| c.is_alphanumeric() || c == '=');
        
        Ok(json!({
            "isvalid": is_valid,
            "address": address,
            "ismine": address == self.wallet.read().await.address
        }))
    }

    async fn estimate_fee(&self, _params: &Value) -> Result<Value, RpcError> {
        // Simple fee estimation - 1000 satoshis
        Ok(json!(1000))
    }
}

// Rocket integration for HTTP RPC server
#[post("/", data = "<request>")]
async fn rpc_handler(
    request: Json<RpcRequest>,
    rpc_server: &State<Arc<RpcServer>>,
) -> Json<RpcResponse> {
    Json(rpc_server.handle_request(request.into_inner()).await)
}

#[get("/")]
fn rpc_info() -> Json<Value> {
    Json(json!({
        "name": "QuantumCoin RPC Server",
        "version": "1.0.0",
        "protocol": "JSON-RPC 2.0",
        "quantum_resistant": true
    }))
}

pub fn build_rpc_rocket(rpc_server: Arc<RpcServer>) -> Rocket<Build> {
    rocket::build()
        .manage(rpc_server)
        .mount("/rpc", routes![rpc_handler, rpc_info])
}

impl Wallet {
    // Helper method for RPC
    fn blocking_read(&self) -> std::sync::RwLockReadGuard<'_, Self> {
        unimplemented!("This should be implemented with proper async handling")
    }
}
