use sqlx::{SqlitePool, Pool, Sqlite, Row};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use thiserror::Error;
use crate::transaction::Transaction;
use crate::block::Block;
use dashmap::DashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use std::path::Path;
use tokio::fs;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Query failed: {0}")]
    QueryFailed(String),
    #[error("Data not found")]
    NotFound,
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
    #[error("IO error: {0}")]
    IoError(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UTXO {
    pub tx_id: String,
    pub output_index: u32,
    pub address: String,
    pub amount: u64,
    pub block_height: u64,
    pub is_spent: bool,
    pub spent_at_height: Option<u64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AddressBalance {
    pub address: String,
    pub confirmed_balance: u64,
    pub unconfirmed_balance: u64,
    pub total_received: u64,
    pub total_sent: u64,
    pub tx_count: u32,
    pub last_updated: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionRecord {
    pub id: String,
    pub block_hash: Option<String>,
    pub block_height: Option<u64>,
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub fee: u64,
    pub status: TransactionStatus,
    pub timestamp: DateTime<Utc>,
    pub confirmations: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Rejected,
}

pub struct ProductionDatabase {
    pool: SqlitePool,
    
    // High-performance in-memory caches
    balance_cache: Arc<DashMap<String, AddressBalance>>,
    utxo_cache: Arc<DashMap<String, UTXO>>,
    block_cache: Arc<DashMap<String, Block>>,
    tx_cache: Arc<DashMap<String, TransactionRecord>>,
    
    // Performance metrics
    cache_stats: Arc<RwLock<CacheStats>>,
}

#[derive(Default, Clone, Debug)]
pub struct CacheStats {
    pub balance_hits: u64,
    pub balance_misses: u64,
    pub utxo_hits: u64,
    pub utxo_misses: u64,
    pub total_queries: u64,
    pub cache_efficiency: f64,
}

impl ProductionDatabase {
    pub async fn new(database_path: &str) -> Result<Self, DatabaseError> {
        // Ensure database directory exists
        if let Some(parent) = Path::new(database_path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await
                    .map_err(|e| DatabaseError::IoError(e.to_string()))?;
            }
        }

        // Connect to SQLite with optimized settings
        let database_url = format!("sqlite://{}?mode=rwc", database_path);
        let pool = SqlitePool::connect(&database_url)
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

        // Configure SQLite for high performance
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
            
        sqlx::query("PRAGMA synchronous = NORMAL")
            .execute(&pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
            
        sqlx::query("PRAGMA cache_size = -64000") // 64MB cache
            .execute(&pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
            
        sqlx::query("PRAGMA temp_store = MEMORY")
            .execute(&pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let db = Self {
            pool,
            balance_cache: Arc::new(DashMap::new()),
            utxo_cache: Arc::new(DashMap::new()),
            block_cache: Arc::new(DashMap::new()),
            tx_cache: Arc::new(DashMap::new()),
            cache_stats: Arc::new(RwLock::new(CacheStats::default())),
        };

        db.initialize_schema().await?;
        db.create_demo_data().await?;
        
        Ok(db)
    }

    async fn initialize_schema(&self) -> Result<(), DatabaseError> {
        // Create blocks table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS blocks (
                hash TEXT PRIMARY KEY,
                height INTEGER UNIQUE NOT NULL,
                previous_hash TEXT,
                merkle_root TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                difficulty INTEGER NOT NULL,
                nonce INTEGER NOT NULL,
                block_data TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_blocks_height ON blocks(height)")
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_blocks_timestamp ON blocks(timestamp)")
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        // Create transactions table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS transactions (
                id TEXT PRIMARY KEY,
                block_hash TEXT,
                block_height INTEGER,
                sender TEXT NOT NULL,
                recipient TEXT NOT NULL,
                amount INTEGER NOT NULL,
                fee INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                timestamp TEXT NOT NULL,
                signature TEXT,
                public_key TEXT,
                nonce INTEGER NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (block_hash) REFERENCES blocks(hash)
            )
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_transactions_sender ON transactions(sender)")
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
            
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_transactions_recipient ON transactions(recipient)")
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
            
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_transactions_block_height ON transactions(block_height)")
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
            
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_transactions_status ON transactions(status)")
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        // Create UTXOs table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS utxos (
                tx_id TEXT NOT NULL,
                output_index INTEGER NOT NULL,
                address TEXT NOT NULL,
                amount INTEGER NOT NULL,
                block_height INTEGER NOT NULL,
                is_spent INTEGER DEFAULT 0,
                spent_at_height INTEGER,
                spent_in_tx TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (tx_id, output_index)
            )
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_utxos_address ON utxos(address)")
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
            
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_utxos_unspent ON utxos(address, is_spent)")
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        // Create address balances table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS address_balances (
                address TEXT PRIMARY KEY,
                confirmed_balance INTEGER NOT NULL DEFAULT 0,
                unconfirmed_balance INTEGER NOT NULL DEFAULT 0,
                total_received INTEGER NOT NULL DEFAULT 0,
                total_sent INTEGER NOT NULL DEFAULT 0,
                tx_count INTEGER NOT NULL DEFAULT 0,
                last_updated TEXT DEFAULT CURRENT_TIMESTAMP
            )
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn create_demo_data(&self) -> Result<(), DatabaseError> {
        // Insert demo addresses with balances
        sqlx::query(r#"
            INSERT OR IGNORE INTO address_balances 
            (address, confirmed_balance, total_received, tx_count)
            VALUES 
            ('QTC1qy8x9s8v7d2k3l4m5n6p7q8r9s0t1u2v3w4x5y6z', 100000000, 100000000, 5),
            ('QTC1qz9y8x7w6v5u4t3s2r1q0p9o8n7m6l5k4j3i2h1g', 50000000, 75000000, 8),
            ('QTC1qa1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t', 25000000, 30000000, 3),
            ('QTC1miner_address_example_wallet_quantum_safe', 1000000000, 1000000000, 100)
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        // Create corresponding UTXOs
        sqlx::query(r#"
            INSERT OR IGNORE INTO utxos 
            (tx_id, output_index, address, amount, block_height, is_spent)
            VALUES 
            ('genesis_tx_1', 0, 'QTC1qy8x9s8v7d2k3l4m5n6p7q8r9s0t1u2v3w4x5y6z', 100000000, 1, 0),
            ('genesis_tx_2', 0, 'QTC1qz9y8x7w6v5u4t3s2r1q0p9o8n7m6l5k4j3i2h1g', 50000000, 1, 0),
            ('genesis_tx_3', 0, 'QTC1qa1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t', 25000000, 1, 0),
            ('miner_reward_1', 0, 'QTC1miner_address_example_wallet_quantum_safe', 1000000000, 2, 0)
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    // Lightning-fast balance lookup with caching
    pub async fn get_balance(&self, address: &str) -> Result<u64, DatabaseError> {
        {
            let mut stats = self.cache_stats.write();
            stats.total_queries += 1;
        }

        // Check in-memory cache first
        if let Some(balance) = self.balance_cache.get(address) {
            {
                let mut stats = self.cache_stats.write();
                stats.balance_hits += 1;
                stats.cache_efficiency = stats.balance_hits as f64 / stats.total_queries as f64;
            }
            return Ok(balance.confirmed_balance);
        }

        // Query database with optimized SQLite query
        {
            let mut stats = self.cache_stats.write();
            stats.balance_misses += 1;
            stats.cache_efficiency = stats.balance_hits as f64 / stats.total_queries as f64;
        }

        let balance = sqlx::query_scalar::<_, i64>(r#"
            SELECT COALESCE(SUM(amount), 0)
            FROM utxos 
            WHERE address = ? AND is_spent = 0
        "#)
        .bind(address)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))? as u64;

        // Cache the result
        let balance_record = AddressBalance {
            address: address.to_string(),
            confirmed_balance: balance,
            unconfirmed_balance: 0,
            total_received: 0,
            total_sent: 0,
            tx_count: 0,
            last_updated: Utc::now(),
        };
        self.balance_cache.insert(address.to_string(), balance_record);

        Ok(balance)
    }

    // High-performance transaction validation
    pub async fn validate_transaction_fast(&self, tx: &Transaction) -> Result<bool, DatabaseError> {
        // Check sender balance (cached)
        let sender_balance = self.get_balance(&tx.sender).await?;
        
        if sender_balance < tx.total_cost() {
            return Ok(false);
        }

        // Check for pending transactions that might affect balance
        let pending_amount = sqlx::query_scalar::<_, i64>(r#"
            SELECT COALESCE(SUM(amount + fee), 0)
            FROM transactions 
            WHERE sender = ? AND status = 'pending'
        "#)
        .bind(&tx.sender)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))? as u64;

        Ok(sender_balance >= pending_amount + tx.total_cost())
    }

    // Batch transaction processing
    pub async fn add_transaction_batch(&self, transactions: &[Transaction]) -> Result<u64, DatabaseError> {
        let mut tx_db = self.pool.begin()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let mut processed = 0u64;

        for transaction in transactions {
            let result = sqlx::query(r#"
                INSERT OR IGNORE INTO transactions 
                (id, sender, recipient, amount, fee, status, timestamp, signature, public_key, nonce)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#)
            .bind(&transaction.id)
            .bind(&transaction.sender)
            .bind(&transaction.recipient)
            .bind(transaction.amount as i64)
            .bind(transaction.fee as i64)
            .bind("pending")
            .bind(transaction.timestamp.to_rfc3339())
            .bind(&transaction.signature)
            .bind(&transaction.public_key)
            .bind(transaction.nonce as i64)
            .execute(&mut *tx_db)
            .await;

            if result.is_ok() && result.unwrap().rows_affected() > 0 {
                processed += 1;
                
                // Invalidate cache for affected addresses
                self.balance_cache.remove(&transaction.sender);
                self.balance_cache.remove(&transaction.recipient);
            }
        }

        tx_db.commit()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(processed)
    }

    // Optimized block insertion
    pub async fn add_block(&self, block: &Block) -> Result<(), DatabaseError> {
        let mut tx_db = self.pool.begin()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        // Insert block
        sqlx::query(r#"
            INSERT OR IGNORE INTO blocks 
            (hash, height, previous_hash, merkle_root, timestamp, difficulty, nonce, block_data, size_bytes)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&block.hash)
        .bind(block.height as i64)
        .bind(&block.previous_hash)
        .bind(&block.merkle_root)
        .bind(block.timestamp.to_rfc3339())
        .bind(block.difficulty as i64)
        .bind(block.nonce as i64)
        .bind(serde_json::to_string(&block).unwrap())
        .bind(block.calculate_size() as i64)
        .execute(&mut *tx_db)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        // Process transactions
        for transaction in &block.transactions {
            // Update transaction status
            sqlx::query(r#"
                UPDATE transactions 
                SET status = 'confirmed', block_hash = ?, block_height = ?
                WHERE id = ?
            "#)
            .bind(&block.hash)
            .bind(block.height as i64)
            .bind(&transaction.id)
            .execute(&mut *tx_db)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

            // Create UTXO for recipient
            sqlx::query(r#"
                INSERT OR IGNORE INTO utxos 
                (tx_id, output_index, address, amount, block_height, is_spent)
                VALUES (?, 0, ?, ?, ?, 0)
            "#)
            .bind(&transaction.id)
            .bind(&transaction.recipient)
            .bind(transaction.amount as i64)
            .bind(block.height as i64)
            .execute(&mut *tx_db)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

            // Mark sender UTXOs as spent (except for mining rewards)
            if transaction.sender != "MINING_REWARD" {
                sqlx::query(r#"
                    UPDATE utxos 
                    SET is_spent = 1, spent_at_height = ?, spent_in_tx = ?
                    WHERE address = ? AND is_spent = 0 
                    AND (SELECT SUM(amount) FROM utxos WHERE address = ? AND is_spent = 0) >= ?
                    LIMIT 1
                "#)
                .bind(block.height as i64)
                .bind(&transaction.id)
                .bind(&transaction.sender)
                .bind(&transaction.sender)
                .bind((transaction.amount + transaction.fee) as i64)
                .execute(&mut *tx_db)
                .await
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
            }

            // Invalidate caches
            self.balance_cache.remove(&transaction.sender);
            self.balance_cache.remove(&transaction.recipient);
        }

        tx_db.commit()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        // Cache the block
        self.block_cache.insert(block.hash.clone(), block.clone());

        Ok(())
    }

    // Transaction history with pagination
    pub async fn get_transaction_history(
        &self,
        address: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<TransactionRecord>, DatabaseError> {
        let rows = sqlx::query(r#"
            SELECT id, block_hash, block_height, sender, recipient, amount, fee, 
                   status, timestamp,
                   CASE 
                       WHEN block_height IS NOT NULL 
                       THEN (SELECT MAX(height) FROM blocks) - block_height + 1
                       ELSE 0 
                   END as confirmations
            FROM transactions 
            WHERE sender = ? OR recipient = ?
            ORDER BY timestamp DESC
            LIMIT ? OFFSET ?
        "#)
        .bind(address)
        .bind(address)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let mut transactions = Vec::new();
        for row in rows {
            let status = match row.get::<&str, _>("status") {
                "pending" => TransactionStatus::Pending,
                "confirmed" => TransactionStatus::Confirmed,
                "failed" => TransactionStatus::Failed,
                "rejected" => TransactionStatus::Rejected,
                _ => TransactionStatus::Pending,
            };

            let timestamp_str: String = row.get("timestamp");
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                .with_timezone(&Utc);

            transactions.push(TransactionRecord {
                id: row.get("id"),
                block_hash: row.get("block_hash"),
                block_height: row.get::<Option<i64>, _>("block_height").map(|h| h as u64),
                sender: row.get("sender"),
                recipient: row.get("recipient"),
                amount: row.get::<i64, _>("amount") as u64,
                fee: row.get::<i64, _>("fee") as u64,
                status,
                timestamp,
                confirmations: row.get::<i64, _>("confirmations") as u32,
            });
        }

        Ok(transactions)
    }

    // Performance monitoring
    pub fn get_cache_stats(&self) -> CacheStats {
        self.cache_stats.read().clone()
    }

    pub async fn get_database_stats(&self) -> Result<HashMap<String, u64>, DatabaseError> {
        let mut stats = HashMap::new();

        let block_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM blocks")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let tx_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM transactions")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let utxo_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM utxos WHERE is_spent = 0")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        stats.insert("total_blocks".to_string(), block_count as u64);
        stats.insert("total_transactions".to_string(), tx_count as u64);
        stats.insert("active_utxos".to_string(), utxo_count as u64);
        stats.insert("cached_balances".to_string(), self.balance_cache.len() as u64);
        stats.insert("cache_efficiency".to_string(), (self.cache_stats.read().cache_efficiency * 100.0) as u64);
        stats.insert("database_size_mb".to_string(), self.get_database_size().await? / 1024 / 1024);

        Ok(stats)
    }

    async fn get_database_size(&self) -> Result<u64, DatabaseError> {
        let size: i64 = sqlx::query_scalar("SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
        
        Ok(size as u64)
    }

    // Maintenance operations
    pub async fn vacuum_database(&self) -> Result<(), DatabaseError> {
        sqlx::query("VACUUM")
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
        
        Ok(())
    }

    pub async fn optimize_database(&self) -> Result<(), DatabaseError> {
        sqlx::query("ANALYZE")
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
        
        Ok(())
    }

    // Cache cleanup
    pub async fn cleanup_cache(&self) {
        let cutoff = Utc::now() - chrono::Duration::minutes(10);
        
        self.balance_cache.retain(|_, balance| {
            balance.last_updated > cutoff
        });
    }
}
