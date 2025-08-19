mod blockchain;
mod transaction;
mod block;
mod mining;
mod mempool;
mod network;
mod revstop;
mod quantum_crypto;
mod wallet;
mod utxo;
mod database;
mod p2p;
mod rpc;
mod explorer;
mod economics;
mod ai_learning;

use blockchain::Blockchain;
use transaction::Transaction;
use block::Block;
use mining::Miner;
use mempool::Mempool;
use network::NetworkNode;
use revstop::RevStop;

use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use tracing::{info, error};
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
    /// Start a full node
    Node {
        /// Port to listen on
        #[arg(short, long, default_value = "8333")]
        port: u16,
        /// Address to bind to
        #[arg(short, long, default_value = "0.0.0.0")]
        bind: String,
        /// Enable mining
        #[arg(short, long)]
        mine: bool,
        /// Mining address
        #[arg(long)]
        mining_address: Option<String>,
        /// Peer addresses to connect to
        #[arg(long)]
        peers: Vec<String>,
    },
    /// Mining operations
    Mine {
        /// Mining address
        address: String,
        /// Number of threads
        #[arg(short, long, default_value = "1")]
        threads: usize,
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
}

#[derive(Subcommand)]
enum BlockchainCommands {
    /// Get blockchain info
    Info,
    /// Get block by hash
    Block { hash: String },
    /// Get transaction by ID
    Transaction { id: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Node { port, bind, mine, mining_address, peers } => {
            start_node(port, &bind, mine, mining_address, peers).await?;
        }
        Commands::Mine { address, threads } => {
            start_mining(&address, threads).await?;
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

async fn start_node(
    port: u16,
    bind: &str,
    enable_mining: bool,
    mining_address: Option<String>,
    peer_addresses: Vec<String>,
) -> Result<()> {
    info!("Starting QuantumCoin node on {}:{}", bind, port);
    
    // Initialize components
    let blockchain = Arc::new(RwLock::new(Blockchain::new()));
    let mempool = Arc::new(RwLock::new(Mempool::default()));
    let revstop = Arc::new(RwLock::new(RevStop::new()));
    
    // Start network node
    let listen_addr: SocketAddr = format!("{}:{}", bind, port).parse()?;
    let mut network_node = NetworkNode::new(listen_addr, {
        let blockchain_read = blockchain.read().await;
        blockchain_read.clone()
    });
    
    network_node.start().await?;
    
    // Connect to peers
    for peer_addr in peer_addresses {
        if let Ok(addr) = peer_addr.parse::<SocketAddr>() {
            if let Err(e) = network_node.connect_to_peer(addr).await {
                error!("Failed to connect to peer {}: {}", addr, e);
            }
        }
    }
    
    // Start mining if enabled
    if enable_mining {
        if let Some(mining_addr) = mining_address {
            let miner = Miner::new(
                mining_addr,
                Arc::clone(&blockchain),
                Arc::clone(&mempool),
                Arc::clone(&revstop),
            );
            
            tokio::spawn(async move {
                if let Err(e) = miner.start_mining().await {
                    error!("Mining error: {}", e);
                }
            });
        } else {
            error!("Mining enabled but no mining address provided");
        }
    }
    
    // Start mempool cleanup task
    {
        let mempool = Arc::clone(&mempool);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
            loop {
                interval.tick().await;
                let mempool_read = mempool.read().await;
                mempool_read.cleanup_expired();
            }
        });
    }
    
    info!("Node started successfully");
    
    // Keep the node running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        
        let peer_count = network_node.get_peer_count().await;
        let mempool_size = {
            let mempool_read = mempool.read().await;
            mempool_read.size()
        };
        let chain_height = {
            let blockchain_read = blockchain.read().await;
            blockchain_read.chain.len()
        };
        
        info!(
            "Node status - Peers: {}, Mempool: {}, Chain height: {}",
            peer_count, mempool_size, chain_height
        );
    }
}

async fn start_mining(address: &str, threads: usize) -> Result<()> {
    info!("Starting mining with {} threads on address: {}", threads, address);
    
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
    
    Ok(())
}

async fn handle_wallet_command(command: WalletCommands) -> Result<()> {
    match command {
        WalletCommands::Generate => {
            let (public_key, private_key) = quantum_crypto::generate_keypair();
            println!("Generated new wallet:");
            println!("Public Key: {}", public_key);
            println!("Private Key: {}", private_key);
            println!("Address: {}", quantum_crypto::public_key_to_address(&public_key));
        }
        WalletCommands::Balance { address } => {
            let blockchain = Blockchain::new();
            let balance = blockchain.get_balance(&address);
            println!("Balance for {}: {} QTC", address, balance as f64 / 100_000_000.0);
        }
        WalletCommands::Send { from, to, amount, fee } => {
            println!("Sending {} QTC from {} to {}", amount as f64 / 100_000_000.0, from, to);
            // TODO: Implement transaction creation and signing
            println!("Transaction functionality not yet implemented in CLI");
        }
    }
    Ok(())
}

async fn handle_blockchain_command(command: BlockchainCommands) -> Result<()> {
    let blockchain = Blockchain::new();
    
    match command {
        BlockchainCommands::Info => {
            println!("Blockchain Info:");
            println!("Height: {}", blockchain.chain.len());
            println!("Difficulty: {}", blockchain.difficulty);
            println!("Total Supply: {} QTC", blockchain.total_supply as f64 / 100_000_000.0);
            println!("Mining Reward: {} QTC", blockchain.get_current_mining_reward() as f64 / 100_000_000.0);
            if let Some(latest_block) = blockchain.chain.last() {
                println!("Latest Block: {}", latest_block.hash);
                println!("Latest Block Time: {}", latest_block.timestamp);
            }
        }
        BlockchainCommands::Block { hash } => {
            if let Some(block) = blockchain.chain.iter().find(|b| b.hash == hash) {
                println!("Block: {}", serde_json::to_string_pretty(block)?);
            } else {
                println!("Block not found: {}", hash);
            }
        }
        BlockchainCommands::Transaction { id } => {
            for block in &blockchain.chain {
                if let Some(tx) = block.transactions.iter().find(|t| t.id == id) {
                    println!("Transaction: {}", serde_json::to_string_pretty(tx)?);
                    return Ok(());
                }
            }
            println!("Transaction not found: {}", id);
        }
    }
    Ok(())
}
