// Test binary for the production P2P networking system
use anyhow::Result;
use quantumcoin::blockchain::Blockchain;
use quantumcoin::network_v2::{NetworkManager, ChainSpec};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🚀 Starting QuantumCoin P2P Network Test");
    
    // Create blockchain instance
    let blockchain = Arc::new(RwLock::new(Blockchain::new()));
    
    // Load chain specification
    let chain_spec = ChainSpec::load_or_default("chain_spec.toml").await;
    
    // Create network manager
    let listen_addr: SocketAddr = "0.0.0.0:8333".parse()?;
    let network_manager = NetworkManager::new(
        listen_addr,
        blockchain,
        Some(chain_spec),
    ).await?;
    
    println!("📡 Starting P2P network on {}", listen_addr);
    
    // Start the P2P network
    network_manager.start().await?;
    
    println!("🔄 Attempting fresh blockchain sync from DNS seeds...");
    
    // Attempt fresh sync
    if let Err(e) = network_manager.sync_from_zero().await {
        println!("⚠️  Initial sync failed: {}", e);
    }
    
    // Wait and show status
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    
    let status = network_manager.get_status().await;
    println!("\n📊 Network Status:");
    println!("   Node ID: {}", status.node_id);
    println!("   Peer Count: {}", status.peer_count);
    println!("   Connected Peers: {:?}", status.connected_peers);
    println!("   Best Height: {}", status.best_height);
    println!("   Sync Progress: {:.1}%", status.sync_progress * 100.0);
    println!("   Uptime: {} seconds", status.uptime);
    
    // Get detailed metrics
    let conn_stats = network_manager.metrics.get_connection_stats().await;
    println!("\n🔌 Connection Stats:");
    println!("   Total Connections: {}", conn_stats.total_connections);
    println!("   Active Connections: {}", conn_stats.active_connections);
    println!("   Connection Failures: {}", conn_stats.connection_failures);
    
    let traffic_stats = network_manager.metrics.get_traffic_stats().await;
    println!("\n📈 Traffic Stats:");
    println!("   Bytes Sent: {}", traffic_stats.bytes_sent);
    println!("   Bytes Received: {}", traffic_stats.bytes_received);
    println!("   Messages Sent: {}", traffic_stats.messages_sent);
    println!("   Messages Received: {}", traffic_stats.messages_received);
    println!("   Bandwidth: {:.2} MB/s", traffic_stats.bandwidth_usage);
    
    let security_stats = network_manager.security_manager.get_security_stats().await;
    println!("\n🔒 Security Stats:");
    println!("   Rejected Connections: {}", security_stats.rejected_connections);
    println!("   Rate Limited IPs: {}", security_stats.rate_limited_ips);
    println!("   Suspicious IPs: {}", security_stats.suspicious_ips);
    println!("   Recent Attacks: {}", security_stats.recent_attacks);
    
    let nat_info = network_manager.nat_manager.get_connection_info().await;
    println!("\n🌐 NAT/Connection Info:");
    println!("   Listen Address: {}", nat_info.listen_address);
    println!("   External Address: {:?}", nat_info.external_address);
    println!("   NAT Type: {:?}", nat_info.nat_type);
    println!("   Supports UPnP: {}", nat_info.supports_upnp);
    println!("   Has Port Mapping: {}", nat_info.has_port_mapping);
    
    // Run for a bit longer
    println!("\n⏳ Running network for 30 seconds...");
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    
    // Export Prometheus metrics
    let prometheus_metrics = network_manager.metrics.export_prometheus().await;
    println!("\n📊 Prometheus Metrics Sample:");
    println!("{}", prometheus_metrics.lines().take(10).collect::<Vec<_>>().join("\n"));
    
    // Graceful shutdown
    println!("\n🛑 Shutting down gracefully...");
    network_manager.shutdown().await?;
    
    println!("✅ QuantumCoin P2P Network Test Complete!");
    Ok(())
}
