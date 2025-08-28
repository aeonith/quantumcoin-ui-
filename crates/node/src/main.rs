mod storage;
mod chainstate;
mod mempool;
mod p2p;
mod pow;
mod rpc;
mod asert;
mod miner;
mod target;

use crate::storage::Storage;
use crate::chainstate::ChainState;
use crate::miner::{build_candidate, mine_block_cpu};
use qc_types::*;
use qc_validation::{ChainSpec, merkle_root, block_subsidy};
use std::{fs, path::PathBuf};
use tracing::{info, error, Level};
use tracing_subscriber::EnvFilter;

fn read_spec(path: &str) -> ChainSpec {
    let content = fs::read_to_string(path).expect("read chain spec");
    toml::from_str(&content).expect("parse chain spec")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .init();

    info!("🚀 Starting QuantumCoin Node");
    info!("⚛️ Post-Quantum Cryptocurrency with RevStop Protection");

    // Load chain specification
    let spec = read_spec("chain_spec.toml");
    info!("📋 Loaded chain spec: {} ({})", spec.network.name, spec.network.symbol);

    // Initialize data directory
    let datadir = PathBuf::from("./.qc-data");
    if !datadir.exists() {
        fs::create_dir_all(&datadir)?;
        info!("📁 Created data directory: {}", datadir.display());
    }

    // Open storage
    let store = Storage::open(&datadir)?;
    info!("💾 Storage initialized");

    let cs = ChainState { spec: &spec, store: &store };

    // Check if we have existing blockchain
    if let Some(tip_hash) = store.get_tip()? {
        info!("📚 Found existing blockchain, tip: {}", tip_hash.to_hex());
    } else {
        info!("🌱 No existing blockchain found, creating genesis block");
        
        // Create genesis block
        let coinbase = Transaction{
            version: 1, 
            lock_time: 0, 
            vin: vec![], // Coinbase has no inputs
            vout: vec![
                TxOut{ 
                    value: block_subsidy(&spec, 0), 
                    kind: OutputType::P2PQRevocable { 
                        pubkey: vec![0u8; 1312], 
                        window_blocks: spec.revstop.window_blocks 
                    } 
                }
            ],
        };
        
        let mut genesis = Block{
            header: BlockHeader{
                version: 1, 
                prev_block: Hash32::zero(),
                merkle_root: merkle_root(&[coinbase.clone()]),
                time: 1_700_000_000, 
                bits: 0x1d00ffff, 
                nonce: 0,
            },
            txs: vec![coinbase],
        };
        
        info!("⛏️ Mining genesis block...");
        if let Some(found) = mine_block_cpu(genesis.clone(), 5_000_000) {
            genesis = found;
        }
        
        cs.apply_block(0, &genesis)?;
        info!("✅ Genesis block applied! Hash: {}", cs.block_hash(&genesis.header).to_hex());
    }

    // Start network services
    info!("🌐 Starting network services...");
    
    // Start RPC server
    tokio::spawn(async {
        if let Err(e) = rpc::serve_rpc().await {
            error!("RPC server error: {}", e);
        }
    });
    
    // Start P2P server  
    tokio::spawn(async {
        if let Err(e) = p2p::run_p2p("0.0.0.0:8333").await {
            error!("P2P server error: {}", e);
        }
    });

    // Mine a few devnet blocks for testing
    info!("⛏️ Mining initial devnet blocks...");
    let mut prev_hash = if let Some(tip) = store.get_tip()? {
        tip
    } else {
        return Err(anyhow::anyhow!("No genesis block found"));
    };

    for height in 1..=5 {
        let coinbase = Transaction{ 
            version: 1, 
            lock_time: 0, 
            vin: vec![],
            vout: vec![ 
                TxOut{ 
                    value: block_subsidy(&spec, height), 
                    kind: OutputType::P2PQ{ pubkey: vec![1u8; 1312] } 
                } 
            ] 
        };
        
        let mut block = build_candidate(prev_hash, 0x1d00ffff, vec![coinbase]);
        
        info!("⛏️ Mining block {}...", height);
        if let Some(found) = mine_block_cpu(block, 10_000_000) { 
            block = found; 
        }
        
        cs.apply_block(height, &block)?;
        prev_hash = cs.block_hash(&block.header);
        
        info!("✅ Mined block {} with hash: {}", height, prev_hash.to_hex());
    }

    info!("🎉 QuantumCoin node startup complete!");
    info!("🔗 RPC server: http://127.0.0.1:8332");
    info!("🌐 P2P listening: 0.0.0.0:8333");
    info!("💡 Try: curl -s http://127.0.0.1:8332/gethealth");

    // Keep node running
    futures::future::pending::<()>().await;
    Ok(())
}
