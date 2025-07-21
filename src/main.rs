use warp::Filter;
use std::sync::{Arc};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use chrono::Utc;
use uuid::Uuid;
use base64::{encode, decode};
use pqcrypto_dilithium::dilithium2::{keypair, sign_detached, verify_detached, PublicKey, SecretKey};
use pqcrypto_traits::sign::{DetachedSignature, PublicKey as TraitPublicKey, SecretKey as TraitSecretKey};

#[derive(Serialize, Deserialize, Clone)]
struct Transaction {
    id: String,
    from: String,
    to: String,
    amount: f64,
    timestamp: String,
    signature: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Block {
    index: usize,
    timestamp: String,
    transactions: Vec<Transaction>,
    prev_hash: String,
    hash: String,
    nonce: u64,
}

#[derive(Default)]
struct Blockchain {
    chain: Vec<Block>,
    mempool: Vec<Transaction>,
    difficulty: usize,
}

impl Blockchain {
    fn new() -> Self {
        let mut bc = Self {
            chain: vec![],
            mempool: vec![],
            difficulty: 4,
        };
        bc.create_genesis_block();
        bc
    }

    fn create_genesis_block(&mut self) {
        let tx = Transaction {
            id: Uuid::new_v4().to_string(),
            from: "GENESIS".into(),
            to: "tNzCy5MT+GQRGlA+JCVIG8juIbmR0MhMSvCP7W0BauzccIB+UKuWBnyOl+nDv91JP2bTkOY30d+tBrlcYZ4wnbELEaNeue4MsLeBATOt0u/z...".into(), // YOUR full pubkey
            amount: 1_250_000.0,
            timestamp: Utc::now().to_rfc3339(),
            signature: None,
        };

        let block = Block {
            index: 0,
            timestamp: Utc::now().to_rfc3339(),
            transactions: vec![tx],
            prev_hash: "0".repeat(64),
            hash: "GENESIS_HASH".to_string(),
            nonce: 0,
        };

        self.chain.push(block);
    }

    fn add_transaction(&mut self, tx: Transaction) {
        self.mempool.push(tx);
    }

    fn mine_block(&mut self) {
        let prev_block = self.chain.last().unwrap();
        let index = prev_block.index + 1;
        let prev_hash = prev_block.hash.clone();
        let transactions = self.mempool.clone();
        self.mempool.clear();

        let mut nonce = 0;
        let mut hash;

        loop {
            let content = format!("{:?}{:?}{:?}{:?}", index, prev_hash, nonce, transactions);
            let digest = sha2::Sha256::digest(content.as_bytes());
            hash = hex::encode(digest);

            if hash.starts_with(&"0".repeat(self.difficulty)) {
                break;
            }

            nonce += 1;
        }

        let block = Block {
            index,
            timestamp: Utc::now().to_rfc3339(),
            transactions,
            prev_hash,
            hash,
            nonce,
        };

        self.chain.push(block);
    }

    fn get_balance(&self, address: &str) -> f64 {
        let mut balance = 0.0;
        for block in &self.chain {
            for tx in &block.transactions {
                if tx.to == address {
                    balance += tx.amount;
                }
                if tx.from == address {
                    balance -= tx.amount;
                }
            }
        }
        balance
    }
}

#[tokio::main]
async fn main() {
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));

    // GET balance
    let balance_route = warp::path!("balance" / String)
        .and(with_blockchain(blockchain.clone()))
        .and_then(get_balance);

    // POST transaction
    let tx_route = warp::path("tx")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_blockchain(blockchain.clone()))
        .and_then(add_transaction);

    // Mine endpoint
    let mine_route = warp::path("mine")
        .and(with_blockchain(blockchain.clone()))
        .and_then(mine_block);

    let routes = balance_route.or(tx_route).or(mine_route);
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}

fn with_blockchain(
    blockchain: Arc<Mutex<Blockchain>>,
) -> impl Filter<Extract = (Arc<Mutex<Blockchain>>,), Error = Infallible> + Clone {
    warp::any().map(move || blockchain.clone())
}

async fn get_balance(address: String, blockchain: Arc<Mutex<Blockchain>>) -> Result<impl warp::Reply, Infallible> {
    let bc = blockchain.lock();
    let balance = bc.get_balance(&address);
    Ok(warp::reply::json(&balance))
}

async fn add_transaction(tx: Transaction, blockchain: Arc<Mutex<Blockchain>>) -> Result<impl warp::Reply, Infallible> {
    let mut bc = blockchain.lock();
    bc.add_transaction(tx.clone());
    Ok(warp::reply::json(&format!("Transaction added: {}", tx.id)))
}

async fn mine_block(blockchain: Arc<Mutex<Blockchain>>) -> Result<impl warp::Reply, Infallible> {
    let mut bc = blockchain.lock();
    bc.mine_block();
    Ok(warp::reply::json(&"Block mined successfully"))
}