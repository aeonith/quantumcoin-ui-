mod blockchain;
mod transaction;
mod block;
mod mining;
mod mempool;
mod network;
mod revstop;
mod quantum_crypto;
mod wallet;
mod merkle;
mod rpc;

use blockchain::Blockchain;
use transaction::Transaction;
use block::Block;
use mining::Miner;
use mempool::Mempool;
use network::{NetworkManager, MiningPool};
use revstop::RevStop;
use wallet::Wallet;
use rpc::{RpcServer, build_rpc_rocket};

use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use tracing::{info, error, warn};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "quantumcoin")]
#[command(about = "QuantumCoin - A quantum-resistant cryptocurrency")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a full node with all services
    Node {
        /// Port to listen on for P2P
        #[arg(short, long, default_value = "8333")]
        port: u16,
        /// Address to bind to
        #[arg(short, long, default_value = "0.0.0.0")]
        bind: String,
        /// RPC port
        #[arg(long, default_value = "8332")]
        rpc_port: u16,
        /// Web UI port
        #[arg(long, default_value = "8080")]
        web_port: u16,
        /// Enable mining
        #[arg(short, long)]
        mine: bool,
        /// Mining address
        #[arg(long)]
        mining_address: Option<String>,
        /// Peer addresses to connect to
        #[arg(long)]
        peers: Vec<String>,
        /// Enable mining pool
        #[arg(long)]
        pool: bool,
    },
    /// Start RPC server only
    RpcServer {
        /// RPC port
        #[arg(short, long, default_value = "8332")]
        port: u16,
    },
    /// Start web interface only
    WebServer {
        /// Web port
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
    /// Mining operations
    Mine {
        /// Mining address
        address: String,
        /// Number of threads
        #[arg(short, long, default_value = "1")]
        threads: usize,
        /// Pool address to connect to
        #[arg(long)]
        pool: Option<String>,
    },
    /// Wallet operations
    Wallet {
        #[command(subcommand)]
        command: WalletCommands,
    },
    /// Blockchain operations
    Blockchain {
        #[command(subcommand)]
        command: BlockchainCommands,
    },
}

#[derive(Subcommand)]
enum WalletCommands {
    /// Generate a new wallet
    Generate,
    /// Get balance
    Balance { address: String },
    /// Send transaction
    Send {
        from: String,
        to: String,
        amount: u64,
        fee: Option<u64>,
    },
    /// List transactions
    List { address: String },
}

#[derive(Subcommand)]
enum BlockchainCommands {
    /// Get blockchain info
    Info,
    /// Get block by hash
    Block { hash: String },
    /// Get transaction by ID
    Transaction { id: String },
    /// Validate blockchain
    Validate,
    /// Sync with network
    Sync { peer: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("quantumcoin=info,debug")
        .init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Node { port, bind, rpc_port, web_port, mine, mining_address, peers, pool } => {
            start_full_node(port, &bind, rpc_port, web_port, mine, mining_address, peers, pool).await?;
        }
        Commands::RpcServer { port } => {
            start_rpc_server(port).await?;
        }
        Commands::WebServer { port } => {
            start_web_server(port).await?;
        }
        Commands::Mine { address, threads, pool } => {
            start_mining(&address, threads, pool).await?;
        }
        Commands::Wallet { command } => {
            handle_wallet_command(command).await?;
        }
        Commands::Blockchain { command } => {
            handle_blockchain_command(command).await?;
        }
    }
    
    Ok(())
}

async fn start_full_node(
    p2p_port: u16,
    bind: &str,
    rpc_port: u16,
    web_port: u16,
    enable_mining: bool,
    mining_address: Option<String>,
    peer_addresses: Vec<String>,
    enable_pool: bool,
) -> Result<()> {
    info!("🚀 Starting QuantumCoin Full Node");
    info!("P2P: {}:{}, RPC: {}, Web: {}", bind, p2p_port, rpc_port, web_port);
    
    // Initialize core components
    let blockchain = Arc::new(RwLock::new(Blockchain::new()));
    let wallet = Arc::new(RwLock::new(Wallet::new()));
    
    // Initialize network
    let listen_addr: SocketAddr = format!("{}:{}", bind, p2p_port).parse()?;
    let network = Arc::new(NetworkManager::new(listen_addr, Arc::clone(&blockchain)));
    
    // Start network
    network.start().await?;
    info!("✅ P2P Network started on {}", listen_addr);
    
    // Connect to bootstrap peers
    for peer_addr in peer_addresses {
        if let Ok(addr) = peer_addr.parse::<SocketAddr>() {
            if let Err(e) = network.connect_to_peer(addr).await {
                warn!("Failed to connect to peer {}: {}", addr, e);
            } else {
                info!("🔗 Connected to peer: {}", addr);
            }
        }
    }
    
    // Sync blockchain
    if network.get_peer_count().await > 0 {
        info!("🔄 Syncing blockchain...");
        network.sync_blockchain().await?;
    }
    
    // Start RPC server
    let rpc_server = Arc::new(
        RpcServer::new(Arc::clone(&blockchain), Arc::clone(&wallet))
            .with_network(Arc::clone(&network))
    );
    
    let rpc_rocket = build_rpc_rocket(Arc::clone(&rpc_server));
    let rpc_config = rocket::Config {
        port: rpc_port,
        address: "0.0.0.0".parse().unwrap(),
        ..rocket::Config::default()
    };
    
    tokio::spawn(async move {
        if let Err(e) = rpc_rocket.configure(rpc_config).launch().await {
            error!("RPC server error: {}", e);
        }
    });
    info!("✅ RPC server started on port {}", rpc_port);
    
    // Start web server (existing backend)
    tokio::spawn(async move {
        let web_rocket = rocket::build()
            .mount("/", rocket::routes![
                backend::index, backend::register, backend::login, 
                backend::kyc_upload, backend::show_keys, backend::toggle_revstop
            ])
            .mount("/static", rocket::fs::FileServer::from("static"))
            .configure(rocket::Config {
                port: web_port,
                address: "0.0.0.0".parse().unwrap(),
                ..rocket::Config::default()
            });
        
        if let Err(e) = web_rocket.launch().await {
            error!("Web server error: {}", e);
        }
    });
    info!("✅ Web interface started on port {}", web_port);
    
    // Initialize mining pool if enabled
    let mining_pool = if enable_pool {
        let pool = Arc::new(MiningPool::new((*network).clone()));
        info!("✅ Mining pool initialized");
        Some(pool)
    } else {
        None
    };
    
    // Start mining if enabled
    if enable_mining {
        if let Some(mining_addr) = mining_address {
            info!("⛏️  Starting mining on address: {}", mining_addr);
            let miner = Miner::new(
                mining_addr,
                Arc::clone(&blockchain),
                Arc::new(RwLock::new(Mempool::default())),
                Arc::new(RwLock::new(RevStop::new())),
            );
            
            tokio::spawn(async move {
                if let Err(e) = miner.start_mining().await {
                    error!("Mining error: {}", e);
                }
            });
        } else {
            error!("⚠️  Mining enabled but no mining address provided");
        }
    }
    
    // Start status reporting
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            
            let peer_count = network.get_peer_count().await;
            let chain_height = {
                let blockchain_read = blockchain.read().await;
                blockchain_read.get_chain_height()
            };
            let total_supply = {
                let blockchain_read = blockchain.read().await;
                blockchain_read.get_total_supply()
            };
            
            info!(
                "📊 Node Status - Peers: {}, Height: {}, Supply: {:.2} QTC",
                peer_count, 
                chain_height, 
                total_supply as f64 / 100_000_000.0
            );
        }
    });
    
    info!("🎉 QuantumCoin Full Node is running!");
    
    // Keep the node running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}

async fn start_rpc_server(port: u16) -> Result<()> {
    info!("Starting QuantumCoin RPC server on port {}", port);
    
    let blockchain = Arc::new(RwLock::new(Blockchain::new()));
    let wallet = Arc::new(RwLock::new(Wallet::new()));
    let rpc_server = Arc::new(RpcServer::new(blockchain, wallet));
    
    let rocket = build_rpc_rocket(rpc_server)
        .configure(rocket::Config {
            port,
            address: "0.0.0.0".parse().unwrap(),
            ..rocket::Config::default()
        });
    
    rocket.launch().await?;
    Ok(())
}

async fn start_web_server(port: u16) -> Result<()> {
    info!("Starting QuantumCoin web interface on port {}", port);
    
    let rocket = rocket::build()
        .mount("/", rocket::routes![
            backend::index, backend::register, backend::login, 
            backend::kyc_upload, backend::show_keys, backend::toggle_revstop
        ])
        .mount("/static", rocket::fs::FileServer::from("static"))
        .configure(rocket::Config {
            port,
            address: "0.0.0.0".parse().unwrap(),
            ..rocket::Config::default()
        });
    
    rocket.launch().await?;
    Ok(())
}

async fn start_mining(
    address: &str, 
    threads: usize, 
    pool_address: Option<String>
) -> Result<()> {
    if let Some(pool_addr) = pool_address {
        info!("Starting pool mining with {} threads, pool: {}", threads, pool_addr);
        // TODO: Implement pool mining client
        unimplemented!("Pool mining not yet implemented");
    } else {
        info!("Starting solo mining with {} threads on address: {}", threads, address);
        
        let blockchain = Arc::new(RwLock::new(Blockchain::new()));
        let mempool = Arc::new(RwLock::new(Mempool::default()));
        let revstop = Arc::new(RwLock::new(RevStop::new()));
        
        let mut miners = Vec::new();
        
        for i in 0..threads {
            let miner = Miner::new(
                format!("{}_{}", address, i),
                Arc::clone(&blockchain),
                Arc::clone(&mempool),
                Arc::clone(&revstop),
            );
            
            let miner_handle = tokio::spawn(async move {
                if let Err(e) = miner.start_mining().await {
                    error!("Miner {} error: {}", i, e);
                }
            });
            
            miners.push(miner_handle);
        }
        
        // Wait for all miners to complete
        for miner_handle in miners {
            let _ = miner_handle.await;
        }
    }
    
    Ok(())
}

async fn handle_wallet_command(command: WalletCommands) -> Result<()> {
    match command {
        WalletCommands::Generate => {
            let wallet = Wallet::new();
            println!("🆕 Generated new wallet:");
            println!("Public Key: {}", wallet.public_key);
            println!("Address: {}", wallet.address);
            println!("⚠️  Save your private key securely!");
        }
        WalletCommands::Balance { address } => {
            let blockchain = Blockchain::new();
            let balance = blockchain.get_balance(&address);
            println!("💰 Balance for {}: {:.8} QTC", address, balance as f64 / 100_000_000.0);
        }
        WalletCommands::Send { from, to, amount, fee } => {
            println!("💸 Sending {:.8} QTC from {} to {}", amount as f64 / 100_000_000.0, from, to);
            // TODO: Implement transaction creation and signing
            println!("⚠️  Transaction functionality requires a running node with RPC");
        }
        WalletCommands::List { address } => {
            let blockchain = Blockchain::new();
            println!("📝 Transactions for address: {}", address);
            
            for (i, block) in blockchain.chain.iter().enumerate() {
                for tx in &block.transactions {
                    if tx.sender == address || tx.recipient == address {
                        let tx_type = if tx.recipient == address { "Received" } else { "Sent" };
                        println!(
                            "Block {}: {} {:.8} QTC - {} (Fee: {:.8})",
                            i,
                            tx_type,
                            tx.amount as f64 / 100_000_000.0,
                            tx.id,
                            tx.fee as f64 / 100_000_000.0
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

async fn handle_blockchain_command(command: BlockchainCommands) -> Result<()> {
    let blockchain = Blockchain::new();
    
    match command {
        BlockchainCommands::Info => {
            println!("⛓️  QuantumCoin Blockchain Info:");
            println!("Height: {}", blockchain.chain.len());
            println!("Difficulty: {}", blockchain.difficulty);
            println!("Total Supply: {:.8} QTC", blockchain.total_supply as f64 / 100_000_000.0);
            println!("Max Supply: {:.8} QTC", blockchain.max_supply as f64 / 100_000_000.0);
            println!("Circulation: {:.2}%", blockchain.get_circulation_percentage());
            println!("Mining Reward: {:.8} QTC", blockchain.get_current_mining_reward() as f64 / 100_000_000.0);
            
            if let Some(latest_block) = blockchain.chain.last() {
                println!("Latest Block: {}", latest_block.hash);
                println!("Latest Block Time: {}", latest_block.timestamp);
                println!("Latest Block Transactions: {}", latest_block.transactions.len());
            }
        }
        BlockchainCommands::Block { hash } => {
            if let Some(block) = blockchain.get_block_by_hash(&hash) {
                println!("📦 Block Details:");
                println!("{}", serde_json::to_string_pretty(block)?);
            } else {
                println!("❌ Block not found: {}", hash);
            }
        }
        BlockchainCommands::Transaction { id } => {
            let mut found = false;
            for block in &blockchain.chain {
                if let Some(tx) = block.transactions.iter().find(|t| t.id == id) {
                    println!("💰 Transaction Details:");
                    println!("{}", serde_json::to_string_pretty(tx)?);
                    found = true;
                    break;
                }
            }
            if !found {
                println!("❌ Transaction not found: {}", id);
            }
        }
        BlockchainCommands::Validate => {
            println!("🔍 Validating blockchain...");
            match blockchain.is_chain_valid().await {
                Ok(true) => println!("✅ Blockchain is valid!"),
                Ok(false) => println!("❌ Blockchain validation failed!"),
                Err(e) => println!("⚠️  Validation error: {}", e),
            }
        }
        BlockchainCommands::Sync { peer } => {
            println!("🔄 Syncing with peer: {}", peer);
            // TODO: Implement peer sync functionality
            println!("⚠️  Sync functionality requires network support");
        }
    }
    Ok(())
}

// Backend routes module (simplified)
mod backend {
    use rocket::{get, post, routes};
    
    #[get("/")]
    pub fn index() -> &'static str {
        "QuantumCoin Node - Visit /static/index.html for the web interface"
    }
    
    #[post("/register")]
    pub fn register() -> &'static str {
        "Registration endpoint"
    }
    
    #[post("/login")]
    pub fn login() -> &'static str {
        "Login endpoint"
    }
    
    #[post("/kyc")]
    pub fn kyc_upload() -> &'static str {
        "KYC upload endpoint"
    }
    
    #[get("/keys")]
    pub fn show_keys() -> &'static str {
        "Wallet keys endpoint"
    }
    
    #[post("/revstop/toggle")]
    pub fn toggle_revstop() -> &'static str {
        "RevStop toggle endpoint"
    }
}
