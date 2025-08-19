# QuantumCoin Operations Guide for Exchanges

## Health & Metrics Endpoints

### Node Health Check
```bash
curl http://127.0.0.1:8332/health
```

**Healthy Response:**
```json
{
  "status": "healthy",
  "uptime_seconds": 86400,
  "version": "1.0.0",
  "network": "mainnet",
  "block_height": 150000,
  "peer_count": 12,
  "mempool_size": 245,
  "sync_progress": 1.0,
  "last_block_time": 1736899800,
  "warnings": []
}
```

**Unhealthy Indicators:**
- `sync_progress < 0.99` (node is syncing)
- `peer_count < 3` (connectivity issues)
- `last_block_time` older than 1 hour (potential network issues)
- Non-empty `warnings` array

### Prometheus Metrics
Endpoint: `http://127.0.0.1:8332/metrics`

**Key Metrics for Alerting:**
```prometheus
# Block production
qtc_blocks_total{chain="main"}
qtc_block_time_seconds{chain="main"}

# Network health  
qtc_peers_connected{chain="main"}
qtc_peer_bans_total{chain="main"}

# Transaction processing
qtc_mempool_size{chain="main"}
qtc_mempool_fee_rate_p50{chain="main"}
qtc_transactions_processed_total{chain="main"}

# Node performance
qtc_node_uptime_seconds
qtc_rpc_requests_total{method="getblock"}
qtc_rpc_request_duration_seconds{method="getblock",quantile="0.95"}

# Storage
qtc_blockchain_size_bytes
qtc_utxo_set_size
qtc_mempool_memory_usage_bytes
```

## Alerting Rules

### Critical Alerts (Page On-Call)

#### Node Down
```yaml
- alert: QuantumCoinNodeDown
  expr: up{job="quantumcoin-node"} == 0
  for: 1m
  labels:
    severity: critical
  annotations:
    summary: "QuantumCoin node is down"
    description: "Node {{ $labels.instance }} has been down for more than 1 minute"
```

#### Chain Stalled
```yaml
- alert: QuantumCoinChainStalled  
  expr: time() - qtc_last_block_time > 3600
  for: 5m
  labels:
    severity: critical
  annotations:
    summary: "QuantumCoin blockchain appears stalled"
    description: "No new blocks for {{ $value | humanizeDuration }}"
```

#### Sync Lost
```yaml
- alert: QuantumCoinSyncLost
  expr: qtc_sync_progress < 0.95
  for: 10m
  labels:
    severity: critical
  annotations:
    summary: "QuantumCoin node lost sync"
    description: "Sync progress is {{ $value | humanizePercentage }}"
```

### Warning Alerts (Monitor)

#### Low Peer Count
```yaml
- alert: QuantumCoinLowPeerCount
  expr: qtc_peers_connected < 5
  for: 15m
  labels:
    severity: warning
  annotations:
    summary: "Low peer count"
    description: "Only {{ $value }} peers connected"
```

#### High Mempool
```yaml
- alert: QuantumCoinHighMempool
  expr: qtc_mempool_size > 5000
  for: 30m
  labels:
    severity: warning
  annotations:
    summary: "High mempool size"
    description: "Mempool has {{ $value }} transactions"
```

#### Reorg Detected
```yaml
- alert: QuantumCoinReorgDetected
  expr: increase(qtc_reorgs_total[1h]) > 0
  labels:
    severity: warning
  annotations:
    summary: "Blockchain reorganization detected"
    description: "{{ $value }} reorgs in the last hour"
```

## Operational Procedures

### Daily Operations

#### Morning Health Check
```bash
#!/bin/bash
# daily_health_check.sh

echo "📊 QuantumCoin Daily Health Check - $(date)"
echo "==========================================="

# Check node status
NODE_HEIGHT=$(curl -s http://127.0.0.1:8332/health | jq -r '.block_height')
PEER_COUNT=$(curl -s http://127.0.0.1:8332/health | jq -r '.peer_count')
MEMPOOL_SIZE=$(curl -s http://127.0.0.1:8332/health | jq -r '.mempool_size')

echo "Block Height: $NODE_HEIGHT"
echo "Peer Count: $PEER_COUNT"
echo "Mempool Size: $MEMPOOL_SIZE"

# Check disk usage
DISK_USAGE=$(df -h /root/.quantumcoin | tail -1 | awk '{print $5}' | sed 's/%//')
echo "Disk Usage: ${DISK_USAGE}%"

# Check recent errors
ERROR_COUNT=$(tail -1000 /var/log/quantumcoin/node.log | grep -c "ERROR\|FATAL" || echo 0)
echo "Recent Errors: $ERROR_COUNT"

# Alert conditions
if [ $PEER_COUNT -lt 5 ]; then
    echo "⚠️  WARNING: Low peer count ($PEER_COUNT)"
fi

if [ $DISK_USAGE -gt 80 ]; then
    echo "⚠️  WARNING: High disk usage (${DISK_USAGE}%)"
fi

if [ $ERROR_COUNT -gt 10 ]; then
    echo "⚠️  WARNING: High error count ($ERROR_COUNT)"
fi

echo "✅ Daily health check complete"
```

### Backup Procedures

#### Blockchain Data Backup
```bash
#!/bin/bash
# backup_blockchain.sh

BACKUP_DIR="/backup/quantumcoin/$(date +%Y%m%d)"
NODE_DATA_DIR="/root/.quantumcoin"

echo "🔄 Starting blockchain backup to $BACKUP_DIR"

# Stop node gracefully
systemctl stop quantumcoin-node

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Backup critical files
cp -r "$NODE_DATA_DIR/blocks" "$BACKUP_DIR/"
cp -r "$NODE_DATA_DIR/chainstate" "$BACKUP_DIR/"
cp "$NODE_DATA_DIR/wallet.dat" "$BACKUP_DIR/"
cp "$NODE_DATA_DIR/quantumcoin.conf" "$BACKUP_DIR/"

# Compress backup
tar -czf "$BACKUP_DIR.tar.gz" -C "$BACKUP_DIR" .
rm -rf "$BACKUP_DIR"

# Restart node
systemctl start quantumcoin-node

echo "✅ Backup complete: $BACKUP_DIR.tar.gz"
```

#### Wallet Backup (Hot Wallets)
```bash
#!/bin/bash
# backup_hot_wallet.sh

WALLET_DIR="/secure/wallets"
BACKUP_DIR="/secure/backups/wallets/$(date +%Y%m%d-%H%M%S)"

mkdir -p "$BACKUP_DIR"

# Backup encrypted wallet files
cp "$WALLET_DIR"/*.dat "$BACKUP_DIR/"

# Create encrypted archive
gpg --cipher-algo AES256 --compress-algo 1 --s2k-mode 3 \
    --s2k-digest-algo SHA512 --s2k-count 65536 \
    --symmetric --output "$BACKUP_DIR.gpg" \
    "$BACKUP_DIR.tar.gz"

# Secure cleanup
shred -vfz -n 3 "$BACKUP_DIR.tar.gz"
rm -rf "$BACKUP_DIR"

echo "✅ Encrypted wallet backup: $BACKUP_DIR.gpg"
```

### Upgrade Procedures

#### Node Upgrade Process
```bash
#!/bin/bash
# upgrade_node.sh

NEW_VERSION="$1"
if [ -z "$NEW_VERSION" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

echo "🔄 Upgrading QuantumCoin node to $NEW_VERSION"

# Download and verify new version
wget "https://github.com/aeonith/quantumcoin-ui-/releases/download/v$NEW_VERSION/quantumcoin-node-linux-x64.tar.gz"
wget "https://github.com/aeonith/quantumcoin-ui-/releases/download/v$NEW_VERSION/SHA256SUMS"
sha256sum -c SHA256SUMS || exit 1

# Backup current version
cp /usr/local/bin/quantumcoin-node "/backup/quantumcoin-node-$(date +%Y%m%d)"

# Stop services gracefully
systemctl stop quantumcoin-indexer
systemctl stop quantumcoin-node

# Install new version
tar -xzf "quantumcoin-node-linux-x64.tar.gz"
cp quantumcoin-node /usr/local/bin/
chmod +x /usr/local/bin/quantumcoin-node

# Restart services
systemctl start quantumcoin-node
sleep 30  # Allow node to start
systemctl start quantumcoin-indexer

# Verify upgrade
NEW_VERSION_RUNNING=$(quantumcoin-node --version | grep -o 'v[0-9.]*')
echo "✅ Upgraded to $NEW_VERSION_RUNNING"

# Health check
sleep 60
curl -f http://127.0.0.1:8332/health || exit 1
echo "✅ Node health check passed"
```

### Incident Response

#### Network Partition Response
1. **Detection**: Multiple peers disconnected, sync progress stuck
2. **Investigation**: Check logs for network errors, verify connectivity
3. **Resolution**: Restart node, check firewall rules, contact upstream
4. **Prevention**: Monitor peer diversity across regions

#### High Memory Usage
1. **Detection**: Memory usage >80% sustained
2. **Investigation**: Check mempool size, peer count, block processing
3. **Resolution**: Restart with pruning enabled, increase memory limits
4. **Prevention**: Set mempool size limits, monitor growth trends

#### Transaction Stuck in Mempool
1. **Detection**: Customer reports transaction not confirming
2. **Investigation**: Check transaction in mempool, verify fee rate
3. **Resolution**: Use RBF if supported, or wait for expiry
4. **Prevention**: Implement dynamic fee estimation

## Snapshot & Restore

### Creating Blockchain Snapshots
```bash
#!/bin/bash
# create_snapshot.sh

SNAPSHOT_HEIGHT="$1"
SNAPSHOT_DIR="/snapshots/quantumcoin"

echo "📸 Creating blockchain snapshot at height $SNAPSHOT_HEIGHT"

# Stop node
systemctl stop quantumcoin-node

# Create snapshot
mkdir -p "$SNAPSHOT_DIR"
tar -czf "$SNAPSHOT_DIR/snapshot-$SNAPSHOT_HEIGHT.tar.gz" \
    -C /root/.quantumcoin blocks chainstate

# Restart node
systemctl start quantumcoin-node

echo "✅ Snapshot created: $SNAPSHOT_DIR/snapshot-$SNAPSHOT_HEIGHT.tar.gz"
```

### Restoring from Snapshot
```bash
#!/bin/bash
# restore_snapshot.sh

SNAPSHOT_FILE="$1"

echo "🔄 Restoring from snapshot: $SNAPSHOT_FILE"

# Stop node
systemctl stop quantumcoin-node

# Backup current state
mv /root/.quantumcoin/blocks "/backup/blocks-$(date +%Y%m%d)"
mv /root/.quantumcoin/chainstate "/backup/chainstate-$(date +%Y%m%d)"

# Restore snapshot
tar -xzf "$SNAPSHOT_FILE" -C /root/.quantumcoin/

# Restart and sync
systemctl start quantumcoin-node

echo "✅ Snapshot restored, node syncing to tip..."
```

## Performance Tuning

### Node Configuration Tuning
```toml
# quantumcoin.conf - Production tuning

# Connection management
maxconnections=125
timeout=5000

# Memory management  
dbcache=4096  # 4GB cache for blockchain DB
maxmempool=2000  # 2GB mempool limit

# Performance
par=4  # Parallel verification threads
checkpoints=1  # Enable checkpoints for faster sync

# Pruning (optional - saves disk space)
prune=50000  # Keep last 50GB of blocks

# Indexing for exchanges
txindex=1
addressindex=1
spentindex=1
timestampindex=1
```

### Database Tuning (PostgreSQL)
```sql
-- postgresql.conf optimizations for indexer DB

# Memory
shared_buffers = 2GB
effective_cache_size = 6GB
work_mem = 256MB
maintenance_work_mem = 1GB

# WAL
wal_buffers = 64MB
checkpoint_completion_target = 0.9
max_wal_size = 4GB

# Query performance
random_page_cost = 1.1  # For SSD storage
effective_io_concurrency = 200

# Logging
log_min_duration_statement = 1000  # Log slow queries
log_checkpoints = on
log_connections = on
log_disconnections = on
```

## Monitoring Dashboard

### Key Performance Indicators (KPIs)

**Uptime Metrics:**
- Node uptime: >99.9%
- RPC response time: <100ms p95
- Block sync lag: <30 seconds

**Network Metrics:**
- Peer connections: 8-50 active peers
- Block propagation time: <10 seconds
- Transaction propagation time: <5 seconds

**Business Metrics:**
- Deposit processing time: <6 confirmations
- Withdrawal success rate: >99.5%
- False positive rate: <0.1%

### Grafana Dashboard JSON
Available at: `D:/quantumcoin-workspace/exchange-pack/monitoring/grafana-dashboard.json`

## Security Checklist

### Node Security
- [ ] RPC access restricted to localhost/private network
- [ ] Strong RPC authentication credentials
- [ ] Regular security updates applied
- [ ] Firewall configured (only P2P port public)
- [ ] Log monitoring for suspicious activity

### Hot Wallet Security  
- [ ] Encrypted wallet files with strong passwords
- [ ] Multi-signature requirements for large withdrawals
- [ ] Regular security audits of signing processes
- [ ] Air-gapped cold storage for reserves
- [ ] Incident response plan documented

### Infrastructure Security
- [ ] TLS encryption for all external communications
- [ ] Regular backup testing and restoration drills
- [ ] Access controls and audit logging
- [ ] Network segmentation and monitoring
- [ ] Dependency scanning and updates

## Troubleshooting

### Common Issues

#### "Node not syncing"
**Symptoms:** `sync_progress` stuck below 1.0
**Causes:** Network connectivity, peer availability, disk space
**Resolution:**
```bash
# Check peer connectivity
quantumcoin-cli getpeerinfo

# Force peer connections
quantumcoin-cli addnode "seed1.quantumcoincrypto.com" "add"

# Check disk space
df -h /root/.quantumcoin
```

#### "High RPC latency"
**Symptoms:** RPC requests taking >1 second
**Causes:** Database locks, high query load, insufficient resources
**Resolution:**
```bash
# Check active RPC connections
netstat -an | grep :8332

# Monitor database performance
tail -f /var/log/postgresql/postgresql.log

# Increase database cache
# Edit postgresql.conf: shared_buffers = 4GB
```

#### "Transaction not confirming"
**Symptoms:** Transaction stuck in mempool >1 hour
**Causes:** Low fee, mempool congestion, invalid transaction
**Resolution:**
```bash
# Check transaction status
quantumcoin-cli getrawtransaction $TXID true

# Check mempool fee rates
quantumcoin-cli getmempoolinfo

# Check if transaction is valid
quantumcoin-cli testmempoolaccept '["$RAW_TX_HEX"]'
```

## Contact & Escalation

### Support Tiers

**Tier 1 - Community Support:**
- Documentation: https://docs.quantumcoincrypto.com
- Forum: https://forum.quantumcoincrypto.com
- Response: Best effort

**Tier 2 - Exchange Support:**
- Email: exchanges@quantumcoincrypto.com
- Response: 24 hours business days
- Covers: Integration issues, configuration help

**Tier 3 - Critical Issues:**
- Email: security@quantumcoincrypto.com
- Response: 4 hours, 24/7
- Covers: Security issues, network problems, consensus failures

### Emergency Contacts
- **CTO**: cto@quantumcoincrypto.com
- **Security Lead**: security@quantumcoincrypto.com  
- **Operations**: ops@quantumcoincrypto.com
- **PGP Keys**: Available in releases and on website

### Escalation Matrix
1. **Service degradation**: Monitor, investigate, document
2. **Service outage**: Page on-call, investigate, restore
3. **Security incident**: Immediate escalation to security team
4. **Data loss**: Immediate escalation to CTO and security team
