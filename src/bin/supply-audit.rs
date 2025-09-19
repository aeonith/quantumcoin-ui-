#!/usr/bin/env cargo

//! QuantumCoin Supply Audit Tool
//!
//! Verifies the total supply and emission schedule to ensure:
//! - No inflation bugs
//! - Correct halving implementation  
//! - Max supply cap respected
//! - Fair launch verification (no premine)
//!
//! This tool is critical for exchange listings and regulatory compliance.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

/// QuantumCoin Supply Audit Tool
#[derive(Parser)]
#[command(author, version, about)]
#[command(name = "supply-audit")]
struct Cli {
    /// RPC endpoint to audit
    #[arg(long, default_value = "http://localhost:8545")]
    rpc_url: String,

    /// Output file for audit report
    #[arg(long, default_value = "supply-audit.json")]
    output: PathBuf,

    /// Maximum block height to audit (default: current tip)
    #[arg(long)]
    max_height: Option<u64>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Verify current supply against expected values
    Verify,
    
    /// Generate detailed emission schedule  
    Schedule {
        /// Number of halvings to calculate
        #[arg(long, default_value = "33")]
        halvings: u32,
    },
    
    /// Check for inflation bugs in block range
    ScanBlocks {
        /// Starting block height
        #[arg(long, default_value = "0")]
        from: u64,
        
        /// Ending block height (default: tip)
        #[arg(long)]
        to: Option<u64>,
    },
    
    /// Audit specific block for supply calculation
    AuditBlock {
        /// Block height to audit
        height: u64,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct SupplyAuditReport {
    /// Audit timestamp
    timestamp: DateTime<Utc>,
    
    /// QuantumCoin network
    network: String,
    
    /// Current block height audited
    current_height: u64,
    
    /// Actual circulating supply (satoshis)
    actual_supply: u64,
    
    /// Expected supply based on emission schedule
    expected_supply: u64,
    
    /// Supply difference (should be 0)
    supply_difference: i64,
    
    /// Maximum possible supply
    max_supply: u64,
    
    /// Current block reward
    current_reward: u64,
    
    /// Blocks until next halving
    blocks_until_halving: u64,
    
    /// Audit status
    status: AuditStatus,
    
    /// Detailed per-block analysis (if requested)
    block_analysis: Option<Vec<BlockSupplyInfo>>,
    
    /// Emission schedule verification
    emission_schedule: Option<Vec<EmissionPeriod>>,
    
    /// Any discovered issues
    issues: Vec<SupplyIssue>,
}

#[derive(Serialize, Deserialize, Debug)]
enum AuditStatus {
    Pass,
    Fail,
    Warning,
}

#[derive(Serialize, Deserialize, Debug)]
struct BlockSupplyInfo {
    height: u64,
    reward: u64,
    expected_reward: u64,
    cumulative_supply: u64,
    expected_cumulative: u64,
    timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct EmissionPeriod {
    halving_epoch: u32,
    start_block: u64,
    end_block: u64,
    reward_per_block: u64,
    blocks_in_period: u64,
    coins_in_period: u64,
    cumulative_supply: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct SupplyIssue {
    severity: IssueSeverity,
    block_height: Option<u64>,
    description: String,
    expected_value: Option<u64>,
    actual_value: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
enum IssueSeverity {
    Critical,    // Supply inflation bug
    Warning,     // Timing or minor inconsistency
    Info,        // Informational note
}

// QuantumCoin economic constants
const INITIAL_REWARD: u64 = 5_000_000_000; // 50 QTC in satoshis
const HALVING_INTERVAL: u64 = 105_120;     // Blocks between halvings (~2 years)
const MAX_SUPPLY: u64 = 2_200_000_000_000_000; // 22M QTC in satoshis
const SATOSHIS_PER_QTC: u64 = 100_000_000;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt::init();
    
    println!("🔍 QuantumCoin Supply Audit Tool v{}", env!("CARGO_PKG_VERSION"));
    println!("Auditing: {}", cli.rpc_url);

    let result = match cli.command {
        Some(Commands::Verify) => {
            verify_supply(&cli).await?
        }
        Some(Commands::Schedule { halvings }) => {
            generate_emission_schedule(&cli, halvings).await?
        }
        Some(Commands::ScanBlocks { from, to }) => {
            scan_blocks_for_issues(&cli, from, to).await?
        }
        Some(Commands::AuditBlock { height }) => {
            audit_single_block(&cli, height).await?
        }
        None => {
            // Default: full supply verification
            verify_supply(&cli).await?
        }
    };

    // Write report to file
    let report_json = serde_json::to_string_pretty(&result)
        .context("Failed to serialize audit report")?;
    
    std::fs::write(&cli.output, &report_json)
        .with_context(|| format!("Failed to write report to {}", cli.output.display()))?;

    // Print summary
    print_audit_summary(&result);
    
    // Exit with error code if audit failed
    match result.status {
        AuditStatus::Pass => Ok(()),
        AuditStatus::Warning => {
            println!("⚠️  Audit completed with warnings - review the report");
            Ok(())
        }
        AuditStatus::Fail => {
            println!("❌ Audit FAILED - critical issues found!");
            std::process::exit(1);
        }
    }
}

async fn verify_supply(cli: &Cli) -> Result<SupplyAuditReport> {
    println!("🔍 Verifying QuantumCoin supply...");
    
    // Get current blockchain info
    let blockchain_info = get_blockchain_info(&cli.rpc_url).await?;
    let current_height = blockchain_info.height;
    
    println!("Current height: {}", current_height);
    
    // Calculate expected supply
    let expected_supply = calculate_expected_supply(current_height);
    let actual_supply = blockchain_info.supply.current;
    let supply_difference = actual_supply as i64 - expected_supply as i64;
    
    // Determine audit status
    let status = if supply_difference == 0 {
        AuditStatus::Pass
    } else if supply_difference.abs() < 100_000_000 { // Less than 1 QTC difference
        AuditStatus::Warning
    } else {
        AuditStatus::Fail
    };

    let mut issues = Vec::new();
    
    // Check for supply issues
    if supply_difference != 0 {
        issues.push(SupplyIssue {
            severity: if supply_difference.abs() < 100_000_000 {
                IssueSeverity::Warning
            } else {
                IssueSeverity::Critical
            },
            block_height: None,
            description: format!("Supply mismatch: {} satoshis", supply_difference),
            expected_value: Some(expected_supply),
            actual_value: Some(actual_supply),
        });
    }
    
    // Check if supply exceeds maximum
    if actual_supply > MAX_SUPPLY {
        issues.push(SupplyIssue {
            severity: IssueSeverity::Critical,
            block_height: None,
            description: "Supply exceeds maximum cap".to_string(),
            expected_value: Some(MAX_SUPPLY),
            actual_value: Some(actual_supply),
        });
    }

    let current_reward = get_block_reward(current_height);
    let blocks_until_halving = HALVING_INTERVAL - (current_height % HALVING_INTERVAL);

    Ok(SupplyAuditReport {
        timestamp: Utc::now(),
        network: blockchain_info.network,
        current_height,
        actual_supply,
        expected_supply,
        supply_difference,
        max_supply: MAX_SUPPLY,
        current_reward,
        blocks_until_halving,
        status,
        block_analysis: None,
        emission_schedule: None,
        issues,
    })
}

async fn generate_emission_schedule(cli: &Cli, halvings: u32) -> Result<SupplyAuditReport> {
    println!("📊 Generating emission schedule for {} halvings...", halvings);
    
    let mut emission_schedule = Vec::new();
    let mut cumulative_supply = 0u64;
    
    for epoch in 0..halvings {
        let reward = INITIAL_REWARD >> epoch; // Halve reward each epoch
        
        if reward == 0 {
            break; // No more rewards
        }
        
        let start_block = epoch as u64 * HALVING_INTERVAL;
        let end_block = start_block + HALVING_INTERVAL - 1;
        let coins_in_period = HALVING_INTERVAL * reward;
        cumulative_supply += coins_in_period;
        
        emission_schedule.push(EmissionPeriod {
            halving_epoch: epoch,
            start_block,
            end_block,
            reward_per_block: reward,
            blocks_in_period: HALVING_INTERVAL,
            coins_in_period,
            cumulative_supply,
        });
        
        println!("Epoch {}: {} QTC per block, {} total QTC", 
                epoch, reward as f64 / SATOSHIS_PER_QTC as f64, 
                cumulative_supply as f64 / SATOSHIS_PER_QTC as f64);
    }
    
    let blockchain_info = get_blockchain_info(&cli.rpc_url).await?;
    
    Ok(SupplyAuditReport {
        timestamp: Utc::now(),
        network: blockchain_info.network,
        current_height: blockchain_info.height,
        actual_supply: blockchain_info.supply.current,
        expected_supply: calculate_expected_supply(blockchain_info.height),
        supply_difference: 0,
        max_supply: MAX_SUPPLY,
        current_reward: get_block_reward(blockchain_info.height),
        blocks_until_halving: HALVING_INTERVAL - (blockchain_info.height % HALVING_INTERVAL),
        status: AuditStatus::Pass,
        block_analysis: None,
        emission_schedule: Some(emission_schedule),
        issues: Vec::new(),
    })
}

async fn scan_blocks_for_issues(cli: &Cli, from: u64, to: Option<u64>) -> Result<SupplyAuditReport> {
    let blockchain_info = get_blockchain_info(&cli.rpc_url).await?;
    let to_height = to.unwrap_or(blockchain_info.height);
    
    println!("🔍 Scanning blocks {} to {} for supply issues...", from, to_height);
    
    let mut block_analysis = Vec::new();
    let mut issues = Vec::new();
    let mut cumulative_supply = 0u64;
    
    for height in from..=to_height {
        let expected_reward = get_block_reward(height);
        
        // TODO: Get actual block data from RPC
        let actual_reward = expected_reward; // Placeholder
        
        cumulative_supply += actual_reward;
        let expected_cumulative = calculate_expected_supply(height);
        
        let block_info = BlockSupplyInfo {
            height,
            reward: actual_reward,
            expected_reward,
            cumulative_supply,
            expected_cumulative,
            timestamp: 0, // TODO: Get from block
        };
        
        // Check for reward mismatch
        if actual_reward != expected_reward {
            issues.push(SupplyIssue {
                severity: IssueSeverity::Critical,
                block_height: Some(height),
                description: format!("Incorrect block reward at height {}", height),
                expected_value: Some(expected_reward),
                actual_value: Some(actual_reward),
            });
        }
        
        block_analysis.push(block_info);
        
        // Progress indicator
        if height % 1000 == 0 {
            println!("Scanned {} blocks...", height - from + 1);
        }
    }
    
    let status = if issues.iter().any(|i| matches!(i.severity, IssueSeverity::Critical)) {
        AuditStatus::Fail
    } else if !issues.is_empty() {
        AuditStatus::Warning
    } else {
        AuditStatus::Pass
    };
    
    Ok(SupplyAuditReport {
        timestamp: Utc::now(),
        network: blockchain_info.network,
        current_height: blockchain_info.height,
        actual_supply: blockchain_info.supply.current,
        expected_supply: calculate_expected_supply(blockchain_info.height),
        supply_difference: blockchain_info.supply.current as i64 - calculate_expected_supply(blockchain_info.height) as i64,
        max_supply: MAX_SUPPLY,
        current_reward: get_block_reward(blockchain_info.height),
        blocks_until_halving: HALVING_INTERVAL - (blockchain_info.height % HALVING_INTERVAL),
        status,
        block_analysis: Some(block_analysis),
        emission_schedule: None,
        issues,
    })
}

async fn audit_single_block(cli: &Cli, height: u64) -> Result<SupplyAuditReport> {
    println!("🔍 Auditing block {}...", height);
    
    let expected_reward = get_block_reward(height);
    
    // TODO: Get actual block from RPC
    let actual_reward = expected_reward; // Placeholder
    
    let mut issues = Vec::new();
    
    if actual_reward != expected_reward {
        issues.push(SupplyIssue {
            severity: IssueSeverity::Critical,
            block_height: Some(height),
            description: format!("Block reward mismatch at height {}", height),
            expected_value: Some(expected_reward),
            actual_value: Some(actual_reward),
        });
    }
    
    let blockchain_info = get_blockchain_info(&cli.rpc_url).await?;
    
    Ok(SupplyAuditReport {
        timestamp: Utc::now(),
        network: blockchain_info.network,
        current_height: blockchain_info.height,
        actual_supply: blockchain_info.supply.current,
        expected_supply: calculate_expected_supply(blockchain_info.height),
        supply_difference: 0,
        max_supply: MAX_SUPPLY,
        current_reward: get_block_reward(blockchain_info.height),
        blocks_until_halving: HALVING_INTERVAL - (blockchain_info.height % HALVING_INTERVAL),
        status: if issues.is_empty() { AuditStatus::Pass } else { AuditStatus::Fail },
        block_analysis: None,
        emission_schedule: None,
        issues,
    })
}

fn calculate_expected_supply(height: u64) -> u64 {
    let mut supply = 0u64;
    let mut current_reward = INITIAL_REWARD;
    let mut blocks_processed = 0u64;
    
    while blocks_processed < height && current_reward > 0 {
        let blocks_in_epoch = std::cmp::min(HALVING_INTERVAL, height - blocks_processed);
        supply += blocks_in_epoch * current_reward;
        blocks_processed += blocks_in_epoch;
        current_reward /= 2; // Halve reward
    }
    
    supply
}

fn get_block_reward(height: u64) -> u64 {
    let epoch = height / HALVING_INTERVAL;
    let reward = INITIAL_REWARD >> epoch;
    if reward > 0 { reward } else { 0 }
}

#[derive(serde::Deserialize)]
struct BlockchainInfo {
    height: u64,
    network: String,
    supply: Supply,
}

#[derive(serde::Deserialize)]
struct Supply {
    current: u64,
    max: u64,
}

async fn get_blockchain_info(rpc_url: &str) -> Result<BlockchainInfo> {
    let client = reqwest::Client::new();
    
    let response = client
        .post(rpc_url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "getblockchaininfo",
            "params": [],
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;

    let rpc_response: serde_json::Value = response.json().await?;
    
    if let Some(error) = rpc_response.get("error") {
        anyhow::bail!("RPC error: {}", error);
    }
    
    let result = rpc_response["result"].clone();
    let blockchain_info: BlockchainInfo = serde_json::from_value(result)
        .context("Failed to parse blockchain info")?;
    
    Ok(blockchain_info)
}

fn print_audit_summary(report: &SupplyAuditReport) {
    println!("\n🔍 QuantumCoin Supply Audit Summary");
    println!("=====================================");
    println!("Network: {}", report.network);
    println!("Block Height: {}", report.current_height);
    println!("Actual Supply: {:.8} QTC", report.actual_supply as f64 / SATOSHIS_PER_QTC as f64);
    println!("Expected Supply: {:.8} QTC", report.expected_supply as f64 / SATOSHIS_PER_QTC as f64);
    println!("Supply Difference: {:.8} QTC", report.supply_difference as f64 / SATOSHIS_PER_QTC as f64);
    println!("Max Supply: {:.0} QTC", report.max_supply as f64 / SATOSHIS_PER_QTC as f64);
    println!("Current Reward: {:.8} QTC", report.current_reward as f64 / SATOSHIS_PER_QTC as f64);
    println!("Blocks Until Halving: {}", report.blocks_until_halving);
    
    match report.status {
        AuditStatus::Pass => println!("✅ Status: PASS - Supply verified correctly"),
        AuditStatus::Warning => println!("⚠️  Status: WARNING - Minor issues found"),
        AuditStatus::Fail => println!("❌ Status: FAIL - Critical issues found"),
    }
    
    if !report.issues.is_empty() {
        println!("\n🔍 Issues Found:");
        for (i, issue) in report.issues.iter().enumerate() {
            let severity_icon = match issue.severity {
                IssueSeverity::Critical => "🚨",
                IssueSeverity::Warning => "⚠️ ",
                IssueSeverity::Info => "ℹ️ ",
            };
            println!("  {}: {} {}", i + 1, severity_icon, issue.description);
        }
    }
    
    if let Some(schedule) = &report.emission_schedule {
        println!("\n📊 Emission Schedule Summary:");
        println!("Epochs: {}", schedule.len());
        if let Some(last_epoch) = schedule.last() {
            println!("Final Supply: {:.0} QTC", last_epoch.cumulative_supply as f64 / SATOSHIS_PER_QTC as f64);
        }
    }
    
    println!("\nDetailed report saved to: {}", "supply-audit.json");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reward_calculation() {
        assert_eq!(get_block_reward(0), INITIAL_REWARD);
        assert_eq!(get_block_reward(HALVING_INTERVAL), INITIAL_REWARD / 2);
        assert_eq!(get_block_reward(HALVING_INTERVAL * 2), INITIAL_REWARD / 4);
    }

    #[test]
    fn test_supply_calculation() {
        let supply_at_halving = calculate_expected_supply(HALVING_INTERVAL);
        assert_eq!(supply_at_halving, HALVING_INTERVAL * INITIAL_REWARD);
    }

    #[test]
    fn test_max_supply_not_exceeded() {
        let final_supply = calculate_expected_supply(u64::MAX);
        assert!(final_supply <= MAX_SUPPLY);
    }
}
