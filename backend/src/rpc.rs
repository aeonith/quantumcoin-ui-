use serde::{Serialize, Deserialize};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::blockchain::{Blockchain, Transaction};
use chrono::Utc;

#[derive(Serialize, Deserialize)]
pub struct RpcRequest {
    pub method: String,
    pub params: serde_json::Value,
    pub id: u64,
}

#[derive(Serialize, Deserialize)]
pub struct RpcResponse {
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub id: u64,
}

pub struct RpcServer {
    blockchain: Arc<RwLock<Blockchain>>,
}

impl RpcServer {
    pub fn new(blockchain: Arc<RwLock<Blockchain>>) -> Self {
        RpcServer { blockchain }
    }

    pub async fn start(&self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        println!("RPC server listening on {}", addr);

        loop {
            let (mut socket, _) = listener.accept().await?;
            let blockchain = Arc::clone(&self.blockchain);

            tokio::spawn(async move {
                let mut buf = vec![0; 1024];
                
                loop {
                    match socket.read(&mut buf).await {
                        Ok(0) => return,
                        Ok(n) => {
                            let request_str = String::from_utf8_lossy(&buf[0..n]);
                            if let Ok(request) = serde_json::from_str::<RpcRequest>(&request_str) {
                                let response = Self::handle_request(request, blockchain.clone()).await;
                                let response_json = serde_json::to_string(&response).unwrap();
                                
                                if socket.write_all(response_json.as_bytes()).await.is_err() {
                                    return;
                                }
                            }
                        }
                        Err(_) => return,
                    }
                }
            });
        }
    }

    async fn handle_request(
        request: RpcRequest,
        blockchain: Arc<RwLock<Blockchain>>,
    ) -> RpcResponse {
        match request.method.as_str() {
            "getblockchain" => {
                let blockchain = blockchain.read().await;
                RpcResponse {
                    result: Some(serde_json::to_value(&blockchain.chain).unwrap()),
                    error: None,
                    id: request.id,
                }
            }
            "getbalance" => {
                if let Some(address) = request.params.get("address").and_then(|v| v.as_str()) {
                    let blockchain = blockchain.read().await;
                    let balance = blockchain.get_balance(address);
                    RpcResponse {
                        result: Some(serde_json::json!({"balance": balance})),
                        error: None,
                        id: request.id,
                    }
                } else {
                    RpcResponse {
                        result: None,
                        error: Some("Address parameter required".to_string()),
                        id: request.id,
                    }
                }
            }
            "sendtransaction" => {
                let params = &request.params;
                if let (Some(from), Some(to), Some(amount)) = (
                    params.get("from").and_then(|v| v.as_str()),
                    params.get("to").and_then(|v| v.as_str()),
                    params.get("amount").and_then(|v| v.as_f64()),
                ) {
                    let transaction = Transaction {
                        id: format!("tx_{}", Utc::now().timestamp()),
                        from: from.to_string(),
                        to: to.to_string(),
                        amount,
                        timestamp: Utc::now(),
                        signature: "".to_string(), // In production, this would be properly signed
                    };

                    let mut blockchain = blockchain.write().await;
                    blockchain.create_transaction(transaction.clone());

                    RpcResponse {
                        result: Some(serde_json::json!({"transaction_id": transaction.id})),
                        error: None,
                        id: request.id,
                    }
                } else {
                    RpcResponse {
                        result: None,
                        error: Some("from, to, and amount parameters required".to_string()),
                        id: request.id,
                    }
                }
            }
            "mineblock" => {
                if let Some(reward_address) = request.params.get("reward_address").and_then(|v| v.as_str()) {
                    let mut blockchain = blockchain.write().await;
                    let mined_block = blockchain.mine_pending_transactions(reward_address.to_string());
                    
                    RpcResponse {
                        result: Some(serde_json::to_value(&mined_block).unwrap()),
                        error: None,
                        id: request.id,
                    }
                } else {
                    RpcResponse {
                        result: None,
                        error: Some("reward_address parameter required".to_string()),
                        id: request.id,
                    }
                }
            }
            _ => RpcResponse {
                result: None,
                error: Some(format!("Unknown method: {}", request.method)),
                id: request.id,
            },
        }
    }
}
