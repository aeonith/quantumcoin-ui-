//! QuantumCoin Genesis Block CLI Tool

use anyhow::{Result, Context};
use clap::{Parser, Subcommand};
use quantumcoin_genesis::{
    GenesisBuilder, GenesisVerifier, ChainSpec, create_mainnet_genesis, create_testnet_genesis,
};
use std::path::PathBuf;
use tracing::{info, warn, error};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "genesis-cli")]
#[command(about = "QuantumCoin Genesis Block Generation and Verification Tool")]
#[command(version = "2.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
    
    /// Output format (json, binary, hex)
    #[arg(short, long, default_value = "json")]
    format: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate mainnet genesis block
    Mainnet {
        /// Output file path
        #[arg(short, long, default_value = "mainnet_genesis.json")]
        output: PathBuf,
        
        /// Custom seed for deterministic generation (hex)
        #[arg(long)]
        seed: Option<String>,
        
        /// Disable deterministic mode
        #[arg(long)]
        non_deterministic: bool,
    },
    
    /// Generate testnet genesis block
    Testnet {
        /// Output file path
        #[arg(short, long, default_value = "testnet_genesis.json")]
        output: PathBuf,
        
        /// Custom seed for deterministic generation (hex)
        #[arg(long)]
        seed: Option<String>,
    },
    
    /// Generate genesis block from custom chain spec
    Custom {
        /// Chain specification file
        #[arg(short, long, default_value = "chain_spec.toml")]
        spec: PathBuf,
        
        /// Output file path
        #[arg(short, long, default_value = "genesis.json")]
        output: PathBuf,
        
        /// Custom seed for deterministic generation (hex)
        #[arg(long)]
        seed: Option<String>,
        
        /// Enable testnet mode
        #[arg(long)]
        testnet: bool,
    },
    
    /// Verify genesis block
    Verify {
        /// Genesis block file to verify
        #[arg(short, long)]
        genesis: PathBuf,
        
        /// Chain specification file
        #[arg(short, long, default_value = "chain_spec.toml")]
        spec: PathBuf,
        
        /// Output detailed verification report
        #[arg(long)]
        detailed: bool,
    },
    
    /// Mine genesis block (for testing different difficulties)
    Mine {
        /// Genesis block file to mine
        #[arg(short, long)]
        genesis: PathBuf,
        
        /// Target difficulty (hex)
        #[arg(short, long)]
        difficulty: Option<String>,
        
        /// Output file for mined block
        #[arg(short, long, default_value = "mined_genesis.json")]
        output: PathBuf,
    },
    
    /// Show genesis block information
    Info {
        /// Genesis block file
        #[arg(short, long)]
        genesis: PathBuf,
    },
    
    /// Reproduce genesis block from existing parameters
    Reproduce {
        /// Genesis block file to reproduce
        #[arg(short, long)]
        genesis: PathBuf,
        
        /// Chain specification file
        #[arg(short, long, default_value = "chain_spec.toml")]
        spec: PathBuf,
        
        /// Output file for reproduced block
        #[arg(short, long, default_value = "reproduced_genesis.json")]
        output: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize tracing
    let subscriber = if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    };
    
    match cli.command {
        Commands::Mainnet { output, seed, non_deterministic } => {
            info!("Generating mainnet genesis block");
            
            let chain_spec = ChainSpec::load_mainnet()
                .context("Failed to load mainnet chain specification")?;
            
            let mut builder = GenesisBuilder::new(chain_spec);
            
            if let Some(seed_hex) = seed {
                let seed_bytes = parse_hex_seed(&seed_hex)?;
                builder = builder.with_seed(seed_bytes);
            }
            
            if non_deterministic {
                builder = builder.deterministic(false);
            }
            
            let genesis = builder.build()
                .context("Failed to build mainnet genesis block")?;
            
            save_genesis_block(&genesis, &output, &cli.format)?;
            
            info!("Mainnet genesis block created:");
            info!("  Hash: {}", genesis.hash_hex());
            info!("  Transactions: {}", genesis.transactions.len());
            info!("  Total allocation: {} QTC", genesis.total_allocation() as f64 / 100_000_000.0);
            info!("  Saved to: {}", output.display());
        }
        
        Commands::Testnet { output, seed } => {
            info!("Generating testnet genesis block");
            
            let chain_spec = ChainSpec::load_testnet()
                .context("Failed to load testnet chain specification")?;
            
            let mut builder = GenesisBuilder::new(chain_spec);
            
            if let Some(seed_hex) = seed {
                let seed_bytes = parse_hex_seed(&seed_hex)?;
                builder = builder.with_seed(seed_bytes);
            }
            
            let genesis = builder.build()
                .context("Failed to build testnet genesis block")?;
            
            save_genesis_block(&genesis, &output, &cli.format)?;
            
            info!("Testnet genesis block created:");
            info!("  Hash: {}", genesis.hash_hex());
            info!("  Transactions: {}", genesis.transactions.len());
            info!("  Allocations: {}", genesis.allocation_transactions().len());
            info!("  Saved to: {}", output.display());
        }
        
        Commands::Custom { spec, output, seed, testnet } => {
            info!("Generating custom genesis block from {}", spec.display());
            
            let mut chain_spec = ChainSpec::load_from_file(&spec)
                .context("Failed to load chain specification")?;
            
            if testnet {
                // Apply testnet modifications
                chain_spec.network_protocol.magic_bytes = [0x51, 0x54, 0x43, 0x54]; // "QTCT"
                chain_spec.consensus.genesis_difficulty = 0x207fffff;
                chain_spec.network_protocol.default_port = 18333;
            }
            
            let mut builder = GenesisBuilder::new(chain_spec);
            
            if let Some(seed_hex) = seed {
                let seed_bytes = parse_hex_seed(&seed_hex)?;
                builder = builder.with_seed(seed_bytes);
            }
            
            let genesis = builder.build()
                .context("Failed to build custom genesis block")?;
            
            save_genesis_block(&genesis, &output, &cli.format)?;
            
            info!("Custom genesis block created:");
            info!("  Hash: {}", genesis.hash_hex());
            info!("  Network: {}", if testnet { "testnet" } else { "custom" });
            info!("  Saved to: {}", output.display());
        }
        
        Commands::Verify { genesis, spec, detailed } => {
            info!("Verifying genesis block {}", genesis.display());
            
            let genesis_block = load_genesis_block(&genesis)?;
            let chain_spec = ChainSpec::load_from_file(&spec)
                .context("Failed to load chain specification")?;
            
            let verifier = GenesisVerifier::new(&chain_spec);
            let result = verifier.verify_detailed(&genesis_block)
                .context("Failed to verify genesis block")?;
            
            if result.valid {
                info!("✓ Genesis block verification PASSED");
            } else {
                error!("✗ Genesis block verification FAILED");
            }
            
            info!("Verification Summary:");
            info!("  Total checks: {}", result.summary.total_checks);
            info!("  Passed: {}", result.summary.passed_checks);
            info!("  Failed: {}", result.summary.failed_checks);
            info!("  Critical failures: {}", result.summary.critical_failures);
            info!("  Warnings: {}", result.summary.warnings);
            
            if detailed {
                info!("\nDetailed Results:");
                for check in &result.checks {
                    let status = if check.passed { "✓" } else { "✗" };
                    let level = match check.severity {
                        quantumcoin_genesis::verification::CheckSeverity::Critical => "CRIT",
                        quantumcoin_genesis::verification::CheckSeverity::Error => "ERR ",
                        quantumcoin_genesis::verification::CheckSeverity::Warning => "WARN",
                        quantumcoin_genesis::verification::CheckSeverity::Info => "INFO",
                    };
                    
                    info!("  {} [{}] {}: {}", status, level, check.name, check.message);
                }
            }
            
            std::process::exit(if result.valid { 0 } else { 1 });
        }
        
        Commands::Mine { genesis, difficulty, output } => {
            info!("Mining genesis block {}", genesis.display());
            
            let mut genesis_block = load_genesis_block(&genesis)?;
            
            if let Some(diff_hex) = difficulty {
                let diff_value = u32::from_str_radix(&diff_hex, 16)
                    .context("Invalid difficulty hex value")?;
                genesis_block.header.difficulty = diff_value;
                info!("Using custom difficulty: 0x{:08x}", diff_value);
            }
            
            info!("Starting proof-of-work mining...");
            quantumcoin_genesis::builder::GenesisMiner::mine_genesis(&mut genesis_block)
                .context("Failed to mine genesis block")?;
            
            save_genesis_block(&genesis_block, &output, &cli.format)?;
            
            info!("Genesis block mined successfully:");
            info!("  Hash: {}", genesis_block.hash_hex());
            info!("  Nonce: {}", genesis_block.header.nonce);
            info!("  Saved to: {}", output.display());
        }
        
        Commands::Info { genesis } => {
            let genesis_block = load_genesis_block(&genesis)?;
            
            println!("Genesis Block Information");
            println!("========================");
            println!("Block Hash: {}", genesis_block.hash_hex());
            println!("Chain Spec Hash: {}", genesis_block.chain_spec_hash_hex());
            println!("Timestamp: {}", genesis_block.header.timestamp);
            println!("Difficulty: 0x{:08x}", genesis_block.header.difficulty);
            println!("Nonce: {}", genesis_block.header.nonce);
            println!("Version: {}", genesis_block.header.version);
            println!("Merkle Root: {}", hex::encode(genesis_block.header.merkle_root));
            println!();
            
            println!("Transactions: {}", genesis_block.transactions.len());
            for (i, tx) in genesis_block.transactions.iter().enumerate() {
                println!("  {}: {} - {} QTC to {}",
                    i,
                    match &tx.tx_type {
                        quantumcoin_genesis::block::TransactionType::Coinbase => "Coinbase",
                        quantumcoin_genesis::block::TransactionType::Allocation { .. } => "Allocation",
                    },
                    tx.amount as f64 / 100_000_000.0,
                    tx.address
                );
            }
            println!();
            
            println!("Total Allocation: {} QTC", genesis_block.total_allocation() as f64 / 100_000_000.0);
            println!("Merkle Tree Depth: {}", genesis_block.merkle_tree.depth);
            
            if let Some(signature) = &genesis_block.signature {
                println!("Signature: {} ({})", 
                    hex::encode(&signature.signature[..32]),
                    signature.algorithm
                );
            } else {
                println!("Signature: None");
            }
            
            println!();
            println!("Metadata:");
            println!("  Creator: {}", genesis_block.metadata.creator);
            println!("  Created: {}", genesis_block.metadata.created_at);
            println!("  Chain Spec Version: {}", genesis_block.metadata.chain_spec_version);
            println!("  Deterministic: {}", genesis_block.metadata.creation_params.deterministic);
        }
        
        Commands::Reproduce { genesis, spec, output } => {
            info!("Reproducing genesis block from {}", genesis.display());
            
            let original_block = load_genesis_block(&genesis)?;
            let chain_spec = ChainSpec::load_from_file(&spec)
                .context("Failed to load chain specification")?;
            
            // Extract parameters from original block
            let creation_params = &original_block.metadata.creation_params;
            
            if !creation_params.deterministic {
                warn!("Original block was not created deterministically - reproduction may not match exactly");
            }
            
            // Recreate using same parameters
            let builder = GenesisBuilder::new(chain_spec)
                .deterministic(creation_params.deterministic);
            
            let reproduced_block = builder.build()
                .context("Failed to reproduce genesis block")?;
            
            // Compare blocks
            let hashes_match = original_block.hash == reproduced_block.hash;
            let merkle_match = original_block.header.merkle_root == reproduced_block.header.merkle_root;
            
            if hashes_match && merkle_match {
                info!("✓ Genesis block reproduced successfully - hashes match");
            } else {
                warn!("⚠ Genesis block reproduction differs from original");
                warn!("  Original hash:    {}", original_block.hash_hex());
                warn!("  Reproduced hash:  {}", reproduced_block.hash_hex());
            }
            
            save_genesis_block(&reproduced_block, &output, &cli.format)?;
            info!("Reproduced block saved to: {}", output.display());
        }
    }
    
    Ok(())
}

/// Parse hex seed string to 32-byte array
fn parse_hex_seed(hex_str: &str) -> Result<[u8; 32]> {
    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    
    if hex_str.len() != 64 {
        anyhow::bail!("Seed must be exactly 64 hex characters (32 bytes)");
    }
    
    let bytes = hex::decode(hex_str)
        .context("Invalid hex string for seed")?;
    
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&bytes);
    Ok(seed)
}

/// Save genesis block in specified format
fn save_genesis_block(
    block: &quantumcoin_genesis::GenesisBlock,
    path: &PathBuf,
    format: &str,
) -> Result<()> {
    match format.to_lowercase().as_str() {
        "json" => {
            let json = block.to_json()?;
            std::fs::write(path, json)?;
        }
        "binary" | "bin" => {
            let bytes = block.to_bytes()?;
            std::fs::write(path, bytes)?;
        }
        "hex" => {
            let bytes = block.to_bytes()?;
            let hex = hex::encode(bytes);
            std::fs::write(path, hex)?;
        }
        _ => anyhow::bail!("Unsupported format: {}", format),
    }
    
    Ok(())
}

/// Load genesis block from file
fn load_genesis_block(path: &PathBuf) -> Result<quantumcoin_genesis::GenesisBlock> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read genesis block file")?;
    
    // Try JSON first
    if let Ok(block) = quantumcoin_genesis::GenesisBlock::from_json(&content) {
        return Ok(block);
    }
    
    // Try binary/hex
    let bytes = if content.chars().all(|c| c.is_ascii_hexdigit() || c.is_whitespace()) {
        // Hex format
        let hex_str: String = content.chars().filter(|c| !c.is_whitespace()).collect();
        hex::decode(hex_str).context("Invalid hex format")?
    } else {
        // Binary format
        std::fs::read(path).context("Failed to read binary file")?
    };
    
    quantumcoin_genesis::GenesisBlock::from_bytes(&bytes)
        .context("Failed to deserialize genesis block")
}
