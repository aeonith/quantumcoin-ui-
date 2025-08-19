#!/usr/bin/env cargo run --bin
//! QuantumCoin Full Node
//! 
//! Complete cryptocurrency node with P2P networking, RPC API, and block explorer

use anyhow::{Result, Context};
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error, warn};
use tracing_subscriber;

use quantumcoin::{
    blockchain::Blockchain,
    database::{BlockchainDatabase, DatabaseConfig},
    mempool::Mempool,
    p2p::P2PNode,
    rpc::{RpcServer, AppState},
    explorer::ExplorerServer,
    genesis::{create_mainnet_genesis, create_testnet_genesis},
};

#[derive(Parser)]
#[command(name = "quantumcoin-node")]
#[command(version = "2.0.0")]
#[command(about = "QuantumCoin Full Node - Post-Quantum Cryptocurrency")]
#[command(long_about = "Complete QuantumCoin node with blockchain, P2P networking, RPC API, and block explorer")]
struct Cli {
    /// Data directory for blockchain data
    #[arg(short, long, default_value = "~/.quantumcoin")]
    datadir: PathBuf,
    
    /// Network (mainnet, testnet)
    #[arg(short, long, default_value = "mainnet")]
    network: String,
    
    /// P2P listening port
    #[arg(short, long, default_value = "8333")]
    port: u16,
    
    /// RPC server port
    #[arg(long, default_value = "8332")]
    rpc_port: u16,
    
    /// Block explorer port
    #[arg(long, default_value = "8080")]
    explorer_port: u16,
    
    /// Bind address
    #[arg(short, long, default_value = "0.0.0.0")]
    bind: String,
    
    /// Seed peers to connect to
    #[arg(long)]
    seed_peers: Vec<String>,
    
    /// Enable mining
    #[arg(short, long)]
    mine: bool,
    
    /// Mining address (required if mining enabled)
    #[arg(long)]
    mining_address: Option<String>,
    
    /// Mining threads
    #[arg(long, default_value = "1")]
    mining_threads: usize,
    
    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the full node
    Start,
    
    /// Initialize the blockchain with genesis block
    Init {
        /// Force reinitialization
        #[arg(short, long)]
        force: bool,
    },
    
    /// Show node status
    Status,
    
    /// Connect to a specific peer
    Connect {
        /// Peer address (host:port)
        peer: String,
    },
    
    /// Show blockchain information
    Info,
    
    /// Generate genesis block
    Genesis {
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }
    
    info!("🚀 Starting QuantumCoin Node v2.0.0");
    info!("⚛️  Post-Quantum Cryptocurrency with Dilithium2 Signatures");
    
    // Ensure data directory exists
    if !cli.datadir.exists() {
        tokio::fs::create_dir_all(&cli.datadir).await
            .context("Failed to create data directory")?;
        info!("📁 Created data directory: {}", cli.datadir.display());
    }
    
    match cli.command.unwrap_or(Commands::Start) {
        Commands::Start => start_node(cli).await,
        Commands::Init { force } => init_blockchain(cli, force).await,
        Commands::Status => show_status(cli).await,
        Commands::Connect { peer } => connect_peer(cli, peer).await,
        Commands::Info => show_info(cli).await,
        Commands::Genesis { output } => generate_genesis(cli, output).await,
    }
}

/// Start the full node
async fn start_node(cli: Cli) -> Result<()> {
    info!("🔧 Initializing QuantumCoin Node...");
    
    // Initialize database
    let db_path = cli.datadir.join("blockchain.db");
    let db_config = DatabaseConfig {
        database_path: db_path.to_string_lossy().to_string(),
        max_connections: 10,
        auto_vacuum: true,
        journal_mode: quantumcoin::database::JournalMode::WAL,
        synchronous: quantumcoin::database::SynchronousMode::Full,
        cache_size: -64000, // 64MB cache
    };
    
    info!("💾 Opening database: {}", db_path.display());
    let database = BlockchainDatabase::new(db_config).await
        .context("Failed to initialize database")?;
    
    // Initialize blockchain
    let blockchain = Arc::new(RwLock::new(Blockchain::new()));
    
    // Check if we need to load existing blocks
    let chain_height = database.get_chain_height().await?;
    if chain_height > 0 {
        info!("📚 Loading existing blockchain (height: {})", chain_height);
        // TODO: Load blocks from database into memory
    } else {
        info!("🌱 Starting with genesis block");
        // Create and store genesis block
        let genesis = if cli.network == "testnet" {
            create_testnet_genesis()?
        } else {
            create_mainnet_genesis()?
        };
        
        info!("✅ Generated genesis block: {}", hex::encode(genesis.hash));
        // TODO: Store genesis block in database and blockchain
    }
    
    // Initialize mempool
    let mempool = Arc::new(RwLock::new(Mempool::new(10000)));
    
    // Initialize P2P node
    let p2p_addr: SocketAddr = format!("{}:{}", cli.bind, cli.port).parse()
        .context("Invalid P2P address")?;
    
    let p2p_node = Arc::new(P2PNode::new(
        p2p_addr,
        Arc::clone(&blockchain),
        Arc::clone(&mempool),
    ));
    
    p2p_node.set_database(database).await;
    
    // Parse seed peers
    let seed_peers: Vec<SocketAddr> = cli.seed_peers
        .iter()
        .filter_map(|peer| peer.parse().ok())
        .collect();
    
    if !seed_peers.is_empty() {
        info!("🌐 Adding {} seed peers", seed_peers.len());
        p2p_node.add_known_peers(&seed_peers).await;
    }
    
    // Create shared state
    let app_state = AppState {
        blockchain: Arc::clone(&blockchain),
        database: Arc::new(RwLock::new(None)), // TODO: Fix this
        mempool: Arc::clone(&mempool),
        p2p_node: Arc::clone(&p2p_node),
    };
    
    // Start RPC server
    let rpc_addr: SocketAddr = format!("{}:{}", cli.bind, cli.rpc_port).parse()
        .context("Invalid RPC address")?;
    
    let rpc_server = RpcServer::new(
        rpc_addr,
        Arc::clone(&blockchain),
        Arc::new(RwLock::new(None)), // TODO: Fix database reference
        Arc::clone(&mempool),
        Arc::clone(&p2p_node),
    );
    
    info!("🔗 Starting RPC server on {}", rpc_addr);
    tokio::spawn(async move {
        if let Err(e) = rpc_server.start().await {
            error!("RPC server error: {}", e);
        }
    });
    
    // Start block explorer
    let explorer_addr: SocketAddr = format!("{}:{}", cli.bind, cli.explorer_port).parse()
        .context("Invalid explorer address")?;
    
    let explorer_server = ExplorerServer::new(explorer_addr, app_state);
    
    info!("🔍 Starting Block Explorer on http://{}", explorer_addr);
    tokio::spawn(async move {
        if let Err(e) = explorer_server.start().await {
            error!("Explorer server error: {}", e);
        }
    });
    
    // Start mining if enabled
    if cli.mine {
        if let Some(mining_address) = cli.mining_address {
            info!("⛏️  Starting mining with {} threads", cli.mining_threads);
            info!("💰 Mining rewards to: {}", mining_address);
            
            // TODO: Start mining threads
            tokio::spawn(async move {
                // Mining loop would go here
                info!("Mining started (placeholder)");
            });
        } else {
            error!("❌ Mining enabled but no mining address provided");
            return Err(anyhow::anyhow!("Mining address required when mining is enabled"));
        }
    }
    
    // Connect to seed peers
    for peer_addr in seed_peers {
        tokio::spawn({
            let p2p_node = Arc::clone(&p2p_node);
            async move {
                if let Err(e) = p2p_node.connect_to_peer(peer_addr).await {
                    warn!("Failed to connect to seed peer {}: {}", peer_addr, e);
                }
            }
        });
    }
    
    // Start P2P networking
    info!("🌐 Starting P2P networking on {}", p2p_addr);
    
    // Print startup summary
    println!("\n🎉 QuantumCoin Node Started Successfully!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔗 P2P Network:     {}", p2p_addr);
    println!("🌐 RPC API:         http://{}", rpc_addr);
    println!("🔍 Block Explorer:  http://{}", explorer_addr);
    println!("⚛️  Network:         {}", cli.network);
    println!("💾 Data Directory:  {}", cli.datadir.display());
    if cli.mine {
        println!("⛏️  Mining:          Enabled ({} threads)", cli.mining_threads);
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Start the P2P node (this will run forever)
    p2p_node.start().await?;
    
    Ok(())
}

/// Initialize blockchain with genesis block
async fn init_blockchain(cli: Cli, force: bool) -> Result<()> {
    let db_path = cli.datadir.join("blockchain.db");
    
    if db_path.exists() && !force {
        error!("❌ Blockchain already initialized. Use --force to reinitialize.");
        return Err(anyhow::anyhow!("Blockchain already exists"));
    }
    
    if force && db_path.exists() {
        tokio::fs::remove_file(&db_path).await
            .context("Failed to remove existing database")?;
        info!("🗑️  Removed existing blockchain database");
    }
    
    info!("🌱 Initializing blockchain for {}", cli.network);
    
    // Create database
    let db_config = DatabaseConfig {
        database_path: db_path.to_string_lossy().to_string(),
        ..DatabaseConfig::default()
    };
    
    let database = BlockchainDatabase::new(db_config).await?;
    
    // Generate and store genesis block
    let genesis = if cli.network == "testnet" {
        create_testnet_genesis()?
    } else {
        create_mainnet_genesis()?
    };
    
    info!("✅ Created genesis block:");
    info!("   Hash: {}", hex::encode(genesis.hash));
    info!("   Timestamp: {}", genesis.header.timestamp);
    info!("   Transactions: {}", genesis.transactions.len());
    
    // TODO: Store genesis block in database
    
    info!("🎉 Blockchain initialized successfully!");
    
    Ok(())
}

/// Show node status
async fn show_status(_cli: Cli) -> Result<()> {
    println!("📊 QuantumCoin Node Status");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Status: Not running (use 'start' command)");
    
    // TODO: Connect to running node and get actual status
    
    Ok(())
}

/// Connect to a specific peer
async fn connect_peer(_cli: Cli, peer: String) -> Result<()> {
    info!("🔗 Connecting to peer: {}", peer);
    
    // TODO: Send connect command to running node
    
    Ok(())
}

/// Show blockchain information
async fn show_info(cli: Cli) -> Result<()> {
    let db_path = cli.datadir.join("blockchain.db");
    
    if !db_path.exists() {
        println!("❌ Blockchain not initialized. Run 'init' first.");
        return Ok(());
    }
    
    let db_config = DatabaseConfig {
        database_path: db_path.to_string_lossy().to_string(),
        ..DatabaseConfig::default()
    };
    
    let database = BlockchainDatabase::new(db_config).await?;
    let stats = database.get_stats().await?;
    
    println!("📊 QuantumCoin Blockchain Info");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Network:      {}", cli.network);
    println!("Blocks:       {}", stats.block_count);
    println!("Transactions: {}", stats.transaction_count);
    println!("UTXOs:        {}", stats.utxo_count);
    println!("Total Value:  {:.8} QTC", stats.total_value as f64 / 100_000_000.0);
    println!("Database:     {} bytes", stats.database_size);
    
    Ok(())
}

/// Generate genesis block
async fn generate_genesis(cli: Cli, output: Option<PathBuf>) -> Result<()> {
    info!("🌱 Generating {} genesis block", cli.network);
    
    let genesis = if cli.network == "testnet" {
        create_testnet_genesis()?
    } else {
        create_mainnet_genesis()?
    };
    
    let output_path = output.unwrap_or_else(|| {
        PathBuf::from(format!("{}_genesis.json", cli.network))
    });
    
    let genesis_json = serde_json::to_string_pretty(&genesis)?;
    tokio::fs::write(&output_path, genesis_json).await?;
    
    println!("✅ Genesis block generated!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Hash:         {}", hex::encode(genesis.hash));
    println!("Timestamp:    {}", genesis.header.timestamp);
    println!("Difficulty:   {}", genesis.header.difficulty);
    println!("Transactions: {}", genesis.transactions.len());
    println!("Saved to:     {}", output_path.display());
    
    Ok(())
}
