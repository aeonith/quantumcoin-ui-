#!/usr/bin/env cargo run --bin
//! QuantumCoin Command Line Interface

use anyhow::{Result, Context};
use clap::{Parser, Subcommand};
use serde_json;
use std::fs;
use std::path::PathBuf;

use quantumcoin::{
    quantum_crypto::{generate_keypair, public_key_to_address, QuantumTransactionSigner},
    transaction::{SignedTransaction, TransactionInput, TransactionOutput},
    utxo::{UTXOSet, UTXO},
    genesis::{create_mainnet_genesis, create_testnet_genesis},
};

#[derive(Parser)]
#[command(name = "quantumcoin-cli")]
#[command(version = "2.0.0")]
#[command(about = "QuantumCoin Command Line Interface")]
#[command(long_about = "A comprehensive CLI for interacting with QuantumCoin blockchain")]
struct Cli {
    /// Data directory
    #[arg(short, long, default_value = "~/.quantumcoin")]
    datadir: PathBuf,
    
    /// Network (mainnet, testnet)
    #[arg(short, long, default_value = "mainnet")]
    network: String,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Wallet operations
    #[command(subcommand)]
    Wallet(WalletCommands),
    
    /// Address operations  
    #[command(subcommand)]
    Address(AddressCommands),
    
    /// Transaction operations
    #[command(subcommand)]
    Transaction(TransactionCommands),
    
    /// Genesis block operations
    #[command(subcommand)]
    Genesis(GenesisCommands),
    
    /// Node operations
    #[command(subcommand)]
    Node(NodeCommands),
    
    /// Network operations
    #[command(subcommand)]
    Network(NetworkCommands),
}

#[derive(Subcommand)]
enum WalletCommands {
    /// Create a new wallet
    Create {
        /// Wallet name
        #[arg(short, long, default_value = "default")]
        name: String,
        
        /// Encrypt with password
        #[arg(short, long)]
        password: Option<String>,
    },
    
    /// List all wallets
    List,
    
    /// Get wallet balance
    Balance {
        /// Wallet name
        #[arg(short, long, default_value = "default")]
        name: String,
    },
    
    /// Send transaction
    Send {
        /// From wallet
        #[arg(short, long, default_value = "default")]
        from: String,
        
        /// Recipient address
        to: String,
        
        /// Amount in QTC (supports decimals)
        amount: f64,
        
        /// Fee in QTC (optional, auto-calculated if not provided)
        #[arg(short, long)]
        fee: Option<f64>,
    },
    
    /// Show wallet addresses
    Addresses {
        /// Wallet name  
        #[arg(short, long, default_value = "default")]
        name: String,
    },
    
    /// Import private key
    Import {
        /// Wallet name
        #[arg(short, long, default_value = "default")]
        name: String,
        
        /// Private key (hex)
        private_key: String,
        
        /// Label for the imported key
        #[arg(short, long)]
        label: Option<String>,
    },
    
    /// Backup wallet
    Backup {
        /// Wallet name
        #[arg(short, long, default_value = "default")]  
        name: String,
        
        /// Backup file path
        output: PathBuf,
    },
    
    /// Restore wallet from backup
    Restore {
        /// Wallet name
        #[arg(short, long, default_value = "default")]
        name: String,
        
        /// Backup file path
        input: PathBuf,
    },
}

#[derive(Subcommand)]
enum AddressCommands {
    /// Generate new address
    New {
        /// Wallet name
        #[arg(short, long, default_value = "default")]
        wallet: String,
        
        /// Label for the address
        #[arg(short, long)]
        label: Option<String>,
    },
    
    /// Validate an address
    Validate {
        /// Address to validate
        address: String,
    },
    
    /// Get address info
    Info {
        /// Address
        address: String,
    },
}

#[derive(Subcommand)]
enum TransactionCommands {
    /// Create a transaction (without broadcasting)
    Create {
        /// From wallet
        #[arg(short, long, default_value = "default")]
        from: String,
        
        /// Recipient address
        to: String,
        
        /// Amount in QTC
        amount: f64,
        
        /// Fee in QTC
        #[arg(short, long)]
        fee: Option<f64>,
        
        /// Output file for transaction
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Sign a transaction
    Sign {
        /// Transaction file
        transaction: PathBuf,
        
        /// Wallet to sign with
        #[arg(short, long, default_value = "default")]
        wallet: String,
    },
    
    /// Broadcast a transaction
    Broadcast {
        /// Signed transaction file
        transaction: PathBuf,
    },
    
    /// Get transaction details
    Info {
        /// Transaction ID
        txid: String,
    },
    
    /// List transactions for a wallet
    List {
        /// Wallet name
        #[arg(short, long, default_value = "default")]
        wallet: String,
        
        /// Number of transactions to show
        #[arg(short, long, default_value = "10")]
        count: usize,
    },
}

#[derive(Subcommand)]
enum GenesisCommands {
    /// Generate genesis block
    Generate {
        /// Network (mainnet/testnet)
        #[arg(short, long, default_value = "mainnet")]
        network: String,
        
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Verify genesis block
    Verify {
        /// Genesis block file
        genesis: PathBuf,
        
        /// Network to verify against
        #[arg(short, long, default_value = "mainnet")]
        network: String,
    },
}

#[derive(Subcommand)]
enum NodeCommands {
    /// Start a node
    Start {
        /// Port to listen on
        #[arg(short, long, default_value = "8333")]
        port: u16,
        
        /// Enable mining
        #[arg(short, long)]
        mine: bool,
        
        /// Mining wallet
        #[arg(long)]
        mining_wallet: Option<String>,
    },
    
    /// Get node info
    Info,
    
    /// Stop the node
    Stop,
}

#[derive(Subcommand)]
enum NetworkCommands {
    /// Get network info
    Info,
    
    /// List connected peers
    Peers,
    
    /// Connect to a peer
    Connect {
        /// Peer address (host:port)
        peer: String,
    },
    
    /// Disconnect from a peer
    Disconnect {
        /// Peer address
        peer: String,
    },
}

/// Wallet storage structure
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Wallet {
    name: String,
    version: String,
    network: String,
    addresses: Vec<WalletAddress>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct WalletAddress {
    address: String,
    public_key: String,
    private_key: String,
    label: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    balance: u64,
    transactions: Vec<String>, // Transaction IDs
}

impl Wallet {
    fn new(name: String, network: String) -> Self {
        Self {
            name,
            version: "2.0.0".to_string(),
            network,
            addresses: Vec::new(),
            created_at: chrono::Utc::now(),
        }
    }
    
    fn add_new_address(&mut self, label: Option<String>) -> &WalletAddress {
        let (public_key, private_key) = generate_keypair();
        let address = public_key_to_address(&public_key);
        
        let wallet_address = WalletAddress {
            address,
            public_key,
            private_key,
            label,
            created_at: chrono::Utc::now(),
            balance: 0,
            transactions: Vec::new(),
        };
        
        self.addresses.push(wallet_address);
        self.addresses.last().unwrap()
    }
    
    fn get_primary_address(&self) -> Option<&WalletAddress> {
        self.addresses.first()
    }
    
    fn total_balance(&self) -> u64 {
        self.addresses.iter().map(|a| a.balance).sum()
    }
    
    fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
    
    fn load_from_file(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let wallet: Wallet = serde_json::from_str(&content)?;
        Ok(wallet)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    if cli.verbose {
        tracing_subscriber::init();
    }
    
    // Ensure data directory exists
    if !cli.datadir.exists() {
        fs::create_dir_all(&cli.datadir)?;
    }
    
    match cli.command {
        Commands::Wallet(cmd) => handle_wallet_command(cmd, &cli).await,
        Commands::Address(cmd) => handle_address_command(cmd, &cli).await,
        Commands::Transaction(cmd) => handle_transaction_command(cmd, &cli).await,
        Commands::Genesis(cmd) => handle_genesis_command(cmd, &cli).await,
        Commands::Node(cmd) => handle_node_command(cmd, &cli).await,
        Commands::Network(cmd) => handle_network_command(cmd, &cli).await,
    }
}

async fn handle_wallet_command(cmd: WalletCommands, cli: &Cli) -> Result<()> {
    match cmd {
        WalletCommands::Create { name, password } => {
            let wallet_path = cli.datadir.join(format!("{}.wallet", name));
            
            if wallet_path.exists() {
                anyhow::bail!("Wallet '{}' already exists", name);
            }
            
            let mut wallet = Wallet::new(name.clone(), cli.network.clone());
            
            // Create initial address
            let initial_address = wallet.add_new_address(Some("Primary".to_string()));
            
            wallet.save_to_file(&wallet_path)?;
            
            println!("✅ Created wallet '{}'", name);
            println!("📍 Initial address: {}", initial_address.address);
            println!("💾 Saved to: {}", wallet_path.display());
            
            if password.is_some() {
                println!("⚠️  Note: Password encryption not yet implemented");
            }
        }
        
        WalletCommands::List => {
            println!("📋 Available wallets:");
            
            let entries = fs::read_dir(&cli.datadir)?;
            let mut count = 0;
            
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                
                if path.extension().and_then(|s| s.to_str()) == Some("wallet") {
                    let name = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");
                    
                    if let Ok(wallet) = Wallet::load_from_file(&path) {
                        println!("  • {} ({})", name, wallet.network);
                        println!("    Addresses: {}", wallet.addresses.len());
                        println!("    Balance: {:.8} QTC", wallet.total_balance() as f64 / 100_000_000.0);
                        count += 1;
                    }
                }
            }
            
            if count == 0 {
                println!("  No wallets found. Use 'wallet create' to create one.");
            }
        }
        
        WalletCommands::Balance { name } => {
            let wallet_path = cli.datadir.join(format!("{}.wallet", name));
            let wallet = Wallet::load_from_file(&wallet_path)
                .context(format!("Failed to load wallet '{}'", name))?;
            
            println!("💰 Wallet '{}' balance:", name);
            println!("  Total: {:.8} QTC", wallet.total_balance() as f64 / 100_000_000.0);
            
            if wallet.addresses.len() > 1 {
                println!("  📍 Address breakdown:");
                for (i, addr) in wallet.addresses.iter().enumerate() {
                    println!("    {}: {:.8} QTC ({})", 
                        i + 1, 
                        addr.balance as f64 / 100_000_000.0,
                        addr.label.as_ref().unwrap_or(&"Unlabeled".to_string())
                    );
                }
            }
        }
        
        WalletCommands::Send { from, to, amount, fee } => {
            let wallet_path = cli.datadir.join(format!("{}.wallet", from));
            let wallet = Wallet::load_from_file(&wallet_path)
                .context(format!("Failed to load wallet '{}'", from))?;
            
            let amount_satoshis = (amount * 100_000_000.0) as u64;
            let fee_satoshis = fee.map(|f| (f * 100_000_000.0) as u64).unwrap_or(100_000); // Default 0.001 QTC fee
            
            // TODO: Implement UTXO selection and transaction creation
            println!("🔄 Creating transaction...");
            println!("  From: {} ({})", from, wallet.get_primary_address().unwrap().address);
            println!("  To: {}", to);
            println!("  Amount: {:.8} QTC", amount);
            println!("  Fee: {:.8} QTC", fee_satoshis as f64 / 100_000_000.0);
            
            println!("⚠️  Transaction creation not yet fully implemented");
            println!("💡 This requires connection to a running node with UTXO index");
        }
        
        WalletCommands::Addresses { name } => {
            let wallet_path = cli.datadir.join(format!("{}.wallet", name));
            let wallet = Wallet::load_from_file(&wallet_path)
                .context(format!("Failed to load wallet '{}'", name))?;
            
            println!("📍 Addresses for wallet '{}':", name);
            
            for (i, addr) in wallet.addresses.iter().enumerate() {
                println!("  {}. {}", i + 1, addr.address);
                println!("     Label: {}", addr.label.as_ref().unwrap_or(&"Unlabeled".to_string()));
                println!("     Balance: {:.8} QTC", addr.balance as f64 / 100_000_000.0);
                println!("     Created: {}", addr.created_at.format("%Y-%m-%d %H:%M:%S"));
                println!();
            }
        }
        
        WalletCommands::Import { name, private_key, label } => {
            let wallet_path = cli.datadir.join(format!("{}.wallet", name));
            let mut wallet = Wallet::load_from_file(&wallet_path)
                .context(format!("Failed to load wallet '{}'", name))?;
            
            // Validate private key and derive public key/address
            match QuantumTransactionSigner::from_private_key(private_key.clone()) {
                Ok(signer) => {
                    let address = signer.get_address();
                    let public_key = signer.get_public_key().to_string();
                    
                    let wallet_address = WalletAddress {
                        address: address.clone(),
                        public_key,
                        private_key,
                        label,
                        created_at: chrono::Utc::now(),
                        balance: 0,
                        transactions: Vec::new(),
                    };
                    
                    wallet.addresses.push(wallet_address);
                    wallet.save_to_file(&wallet_path)?;
                    
                    println!("✅ Imported address: {}", address);
                    println!("💾 Updated wallet '{}'", name);
                }
                Err(e) => {
                    anyhow::bail!("Invalid private key: {}", e);
                }
            }
        }
        
        WalletCommands::Backup { name, output } => {
            let wallet_path = cli.datadir.join(format!("{}.wallet", name));
            let wallet = Wallet::load_from_file(&wallet_path)
                .context(format!("Failed to load wallet '{}'", name))?;
            
            wallet.save_to_file(&output)?;
            println!("✅ Wallet '{}' backed up to: {}", name, output.display());
            println!("⚠️  Keep this backup file secure - it contains private keys!");
        }
        
        WalletCommands::Restore { name, input } => {
            let wallet_path = cli.datadir.join(format!("{}.wallet", name));
            
            if wallet_path.exists() {
                anyhow::bail!("Wallet '{}' already exists", name);
            }
            
            let mut wallet = Wallet::load_from_file(&input)
                .context("Failed to load backup file")?;
            
            wallet.name = name.clone(); // Update name
            wallet.save_to_file(&wallet_path)?;
            
            println!("✅ Wallet '{}' restored from backup", name);
            println!("📍 {} addresses restored", wallet.addresses.len());
        }
    }
    
    Ok(())
}

async fn handle_address_command(cmd: AddressCommands, cli: &Cli) -> Result<()> {
    match cmd {
        AddressCommands::New { wallet, label } => {
            let wallet_path = cli.datadir.join(format!("{}.wallet", wallet));
            let mut wallet_data = Wallet::load_from_file(&wallet_path)
                .context(format!("Failed to load wallet '{}'", wallet))?;
            
            let new_address = wallet_data.add_new_address(label.clone());
            wallet_data.save_to_file(&wallet_path)?;
            
            println!("✅ Generated new address: {}", new_address.address);
            if let Some(label) = label {
                println!("🏷️  Label: {}", label);
            }
        }
        
        AddressCommands::Validate { address } => {
            // TODO: Implement proper address validation
            if address.starts_with("qtc1q") && address.len() >= 42 {
                println!("✅ Address appears valid: {}", address);
            } else {
                println!("❌ Invalid address format: {}", address);
            }
        }
        
        AddressCommands::Info { address } => {
            println!("📍 Address: {}", address);
            println!("⚠️  Address info lookup requires connection to node");
            println!("💡 Use 'node start' to enable blockchain queries");
        }
    }
    
    Ok(())
}

async fn handle_transaction_command(cmd: TransactionCommands, _cli: &Cli) -> Result<()> {
    match cmd {
        TransactionCommands::Create { .. } => {
            println!("⚠️  Transaction creation not yet implemented");
            println!("💡 This requires UTXO indexing and node connection");
        }
        
        TransactionCommands::Sign { .. } => {
            println!("⚠️  Transaction signing not yet implemented");
        }
        
        TransactionCommands::Broadcast { .. } => {
            println!("⚠️  Transaction broadcasting not yet implemented");
            println!("💡 This requires connection to network peers");
        }
        
        TransactionCommands::Info { txid } => {
            println!("🔍 Looking up transaction: {}", txid);
            println!("⚠️  Transaction lookup requires node connection");
        }
        
        TransactionCommands::List { wallet, count } => {
            println!("📋 Recent transactions for wallet '{}' ({})", wallet, count);
            println!("⚠️  Transaction history requires blockchain indexing");
        }
    }
    
    Ok(())
}

async fn handle_genesis_command(cmd: GenesisCommands, _cli: &Cli) -> Result<()> {
    match cmd {
        GenesisCommands::Generate { network, output } => {
            println!("🪙 Generating {} genesis block...", network);
            
            let genesis = match network.as_str() {
                "mainnet" => create_mainnet_genesis()?,
                "testnet" => create_testnet_genesis()?,
                _ => anyhow::bail!("Unknown network: {}", network),
            };
            
            let output_path = output.unwrap_or_else(|| {
                PathBuf::from(format!("{}_genesis.json", network))
            });
            
            let genesis_json = serde_json::to_string_pretty(&genesis)?;
            fs::write(&output_path, genesis_json)?;
            
            println!("✅ Genesis block created!");
            println!("📊 Block hash: {}", hex::encode(genesis.hash));
            println!("🌱 Merkle root: {}", hex::encode(genesis.header.merkle_root));
            println!("⏰ Timestamp: {}", genesis.header.timestamp);
            println!("💾 Saved to: {}", output_path.display());
        }
        
        GenesisCommands::Verify { genesis, network } => {
            println!("🔍 Verifying genesis block for {}...", network);
            
            let content = fs::read_to_string(genesis)?;
            let genesis_block: serde_json::Value = serde_json::from_str(&content)?;
            
            println!("✅ Genesis block format is valid");
            println!("⚠️  Full cryptographic verification not yet implemented");
        }
    }
    
    Ok(())
}

async fn handle_node_command(cmd: NodeCommands, _cli: &Cli) -> Result<()> {
    match cmd {
        NodeCommands::Start { port, mine, mining_wallet } => {
            println!("🚀 Starting QuantumCoin node on port {}...", port);
            
            if mine {
                if let Some(wallet) = mining_wallet {
                    println!("⛏️  Mining enabled for wallet: {}", wallet);
                } else {
                    println!("⚠️  Mining enabled but no wallet specified");
                }
            }
            
            println!("⚠️  Full node implementation not yet complete");
            println!("💡 This will start P2P networking, blockchain sync, and mining");
        }
        
        NodeCommands::Info => {
            println!("ℹ️  Node info:");
            println!("⚠️  Requires running node instance");
        }
        
        NodeCommands::Stop => {
            println!("⏹️  Stopping node...");
            println!("⚠️  Node management not yet implemented");
        }
    }
    
    Ok(())
}

async fn handle_network_command(cmd: NetworkCommands, _cli: &Cli) -> Result<()> {
    match cmd {
        NetworkCommands::Info => {
            println!("🌐 Network info:");
            println!("  Name: QuantumCoin");
            println!("  Symbol: QTC");
            println!("  Decimals: 8");
            println!("  Max Supply: 22,000,000 QTC");
            println!("  Block Time: 10 minutes");
            println!("  Algorithm: Proof of Work (Blake3)");
            println!("  Signatures: Dilithium2 (Post-Quantum)");
            println!("⚠️  Live network stats require node connection");
        }
        
        NetworkCommands::Peers => {
            println!("👥 Connected peers:");
            println!("⚠️  Peer information requires running node");
        }
        
        NetworkCommands::Connect { peer } => {
            println!("🔗 Connecting to peer: {}", peer);
            println!("⚠️  Peer connection requires running node");
        }
        
        NetworkCommands::Disconnect { peer } => {
            println!("❌ Disconnecting from peer: {}", peer);
            println!("⚠️  Peer management requires running node");
        }
    }
    
    Ok(())
}
