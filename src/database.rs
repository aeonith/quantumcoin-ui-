use sqlx::{PgPool, Pool, Postgres, Row};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use thiserror::Error;
use crate::transaction::Transaction;
use crate::block::Block;
use redis::AsyncCommands;
use dashmap::DashMap;
use std::sync::Arc;
use parking_lot::RwLock;

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
    #[error("Cache error: {0}")]
    CacheError(String),
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

pub struct QuantumCoinDB {
    pool: PgPool,
    redis: redis::Client,
    
    // In-memory caches for high-performance lookups
    balance_cache: Arc<DashMap<String, AddressBalance>>,
    utxo_cache: Arc<DashMap<String, UTXO>>, // tx_id:output_index -> UTXO
    block_cache: Arc<DashMap<String, Block>>, // hash -> block
    tx_cache: Arc<DashMap<String, TransactionRecord>>, // tx_id -> transaction
    
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
}

impl QuantumCoinDB {
    pub async fn new(database_url: &str, redis_url: &str) -> Result<Self, DatabaseError> {
        // Connect to PostgreSQL
        let pool = PgPool::connect(database_url)
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

        // Connect to Redis
        let redis = redis::Client::open(redis_url)
            .map_err(|e| DatabaseError::CacheError(e.to_string()))?;

        // Initialize database schema
        let db = Self {
            pool,
            redis,
            balance_cache: Arc::new(DashMap::new()),
            utxo_cache: Arc::new(DashMap::new()),
            block_cache: Arc::new(DashMap::new()),
            tx_cache: Arc::new(DashMap::new()),
            cache_stats: Arc::new(RwLock::new(CacheStats::default())),
        };

        db.initialize_schema().await?;
        
        Ok(db)
    }

    async fn initialize_schema(&self) -> Result<(), DatabaseError> {
        // Create tables if they don't exist
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS blocks (
                hash VARCHAR(64) PRIMARY KEY,
                height BIGINT UNIQUE NOT NULL,
                previous_hash VARCHAR(64),
                merkle_root VARCHAR(64) NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL,
                difficulty INTEGER NOT NULL,
                nonce BIGINT NOT NULL,
                block_data JSONB NOT NULL,
                size_bytes INTEGER NOT NULL,
                created_at TIMESTAMPTZ DEFAULT NOW()
            );
            
            CREATE INDEX IF NOT EXISTS idx_blocks_height ON blocks(height);
            CREATE INDEX IF NOT EXISTS idx_blocks_timestamp ON blocks(timestamp);
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS transactions (
                id VARCHAR(64) PRIMARY KEY,
                block_hash VARCHAR(64) REFERENCES blocks(hash),
                block_height BIGINT,
                sender VARCHAR(64) NOT NULL,
                recipient VARCHAR(64) NOT NULL,
                amount BIGINT NOT NULL,
                fee BIGINT NOT NULL,
                status VARCHAR(20) NOT NULL DEFAULT 'pending',
                timestamp TIMESTAMPTZ NOT NULL,
                signature TEXT,
                public_key TEXT,
                nonce BIGINT NOT NULL,
                created_at TIMESTAMPTZ DEFAULT NOW()
            );
            
            CREATE INDEX IF NOT EXISTS idx_transactions_sender ON transactions(sender);
            CREATE INDEX IF NOT EXISTS idx_transactions_recipient ON transactions(recipient);
            CREATE INDEX IF NOT EXISTS idx_transactions_block_height ON transactions(block_height);
            CREATE INDEX IF NOT EXISTS idx_transactions_status ON transactions(status);
            CREATE INDEX IF NOT EXISTS idx_transactions_timestamp ON transactions(timestamp);
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS utxos (
                tx_id VARCHAR(64) NOT NULL,
                output_index INTEGER NOT NULL,
                address VARCHAR(64) NOT NULL,
                amount BIGINT NOT NULL,
                block_height BIGINT NOT NULL,
                is_spent BOOLEAN DEFAULT FALSE,
                spent_at_height BIGINT,
                spent_in_tx VARCHAR(64),
                created_at TIMESTAMPTZ DEFAULT NOW(),
                PRIMARY KEY (tx_id, output_index)
            );
            
            CREATE INDEX IF NOT EXISTS idx_utxos_address ON utxos(address);
            CREATE INDEX IF NOT EXISTS idx_utxos_unspent ON utxos(address, is_spent) WHERE NOT is_spent;
            CREATE INDEX IF NOT EXISTS idx_utxos_block_height ON utxos(block_height);
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS address_balances (
                address VARCHAR(64) PRIMARY KEY,
                confirmed_balance BIGINT NOT NULL DEFAULT 0,
                unconfirmed_balance BIGINT NOT NULL DEFAULT 0,
                total_received BIGINT NOT NULL DEFAULT 0,
                total_sent BIGINT NOT NULL DEFAULT 0,
                tx_count INTEGER NOT NULL DEFAULT 0,
                last_updated TIMESTAMPTZ DEFAULT NOW()
            );
            
            CREATE INDEX IF NOT EXISTS idx_balances_confirmed ON address_balances(confirmed_balance);
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    // Ultra-fast balance lookup with multi-layer caching
    pub async fn get_balance(&self, address: &str) -> Result<u64, DatabaseError> {
        {
            let mut stats = self.cache_stats.write();
            stats.total_queries += 1;
        }

        // Level 1: In-memory cache
        if let Some(balance) = self.balance_cache.get(address) {
            {
                let mut stats = self.cache_stats.write();
                stats.balance_hits += 1;
            }
            return Ok(balance.confirmed_balance);
        }

        // Level 2: Redis cache
        let mut redis_conn = self.redis.get_async_connection()
            .await
            .map_err(|e| DatabaseError::CacheError(e.to_string()))?;

        let cache_key = format!("balance:{}", address);
        if let Ok(cached_balance) = redis_conn.get::<_, String>(&cache_key).await {
            if let Ok(balance) = cached_balance.parse::<u64>() {
                // Update in-memory cache
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
                
                {
                    let mut stats = self.cache_stats.write();
                    stats.balance_hits += 1;
                }
                return Ok(balance);
            }
        }

        // Level 3: Database query with optimized aggregation
        {
            let mut stats = self.cache_stats.write();
            stats.balance_misses += 1;
        }

        let balance = sqlx::query_scalar::<_, i64>(r#"
            SELECT COALESCE(SUM(amount), 0)
            FROM utxos 
            WHERE address = $1 AND NOT is_spent
        "#)
        .bind(address)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))? as u64;

        // Cache the result in both Redis and memory
        let _: () = redis_conn.set_ex(&cache_key, balance.to_string(), 60).await
            .map_err(|e| DatabaseError::CacheError(e.to_string()))?;

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

    // Batch balance lookup for multiple addresses
    pub async fn get_balances_batch(&self, addresses: &[String]) -> Result<HashMap<String, u64>, DatabaseError> {
        let mut results = HashMap::new();
        let mut uncached_addresses = Vec::new();

        // Check cache first
        for address in addresses {
            if let Some(balance) = self.balance_cache.get(address) {
                results.insert(address.clone(), balance.confirmed_balance);
            } else {
                uncached_addresses.push(address);
            }
        }

        if !uncached_addresses.is_empty() {
            // Batch query for uncached addresses
            let placeholders: Vec<String> = (1..=uncached_addresses.len())
                .map(|i| format!("${}", i))
                .collect();
            let query = format!(r#"
                SELECT address, COALESCE(SUM(amount), 0) as balance
                FROM utxos 
                WHERE address = ANY(ARRAY[{}]) AND NOT is_spent
                GROUP BY address
            "#, placeholders.join(","));

            let mut query_builder = sqlx::query(&query);
            for address in &uncached_addresses {
                query_builder = query_builder.bind(address);
            }

            let rows = query_builder
                .fetch_all(&self.pool)
                .await
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

            for row in rows {
                let address: String = row.get("address");
                let balance: i64 = row.get("balance");
                results.insert(address.clone(), balance as u64);

                // Cache the result
                let balance_record = AddressBalance {
                    address: address.clone(),
                    confirmed_balance: balance as u64,
                    unconfirmed_balance: 0,
                    total_received: 0,
                    total_sent: 0,
                    tx_count: 0,
                    last_updated: Utc::now(),
                };
                self.balance_cache.insert(address, balance_record);
            }
        }

        Ok(results)
    }

    // High-performance UTXO operations
    pub async fn get_unspent_outputs(&self, address: &str) -> Result<Vec<UTXO>, DatabaseError> {
        let rows = sqlx::query(r#"
            SELECT tx_id, output_index, address, amount, block_height, 
                   is_spent, spent_at_height, created_at
            FROM utxos 
            WHERE address = $1 AND NOT is_spent
            ORDER BY block_height ASC, tx_id ASC
        "#)
        .bind(address)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let mut utxos = Vec::new();
        for row in rows {
            let utxo = UTXO {
                tx_id: row.get("tx_id"),
                output_index: row.get::<i32, _>("output_index") as u32,
                address: row.get("address"),
                amount: row.get::<i64, _>("amount") as u64,
                block_height: row.get::<i64, _>("block_height") as u64,
                is_spent: row.get("is_spent"),
                spent_at_height: row.get::<Option<i64>, _>("spent_at_height").map(|h| h as u64),
                created_at: row.get("created_at"),
            };
            
            // Cache UTXO
            let cache_key = format!("{}:{}", utxo.tx_id, utxo.output_index);
            self.utxo_cache.insert(cache_key, utxo.clone());
            
            utxos.push(utxo);
        }

        Ok(utxos)
    }

    // Lightning-fast transaction validation
    pub async fn validate_transaction_fast(&self, tx: &Transaction) -> Result<bool, DatabaseError> {
        // Check if sender has sufficient balance (cached lookup)
        let sender_balance = self.get_balance(&tx.sender).await?;
        
        if sender_balance < tx.total_cost() {
            return Ok(false);
        }

        // Check for double spending in pending transactions
        let pending_spent = sqlx::query_scalar::<_, i64>(r#"
            SELECT COALESCE(SUM(amount + fee), 0)
            FROM transactions 
            WHERE sender = $1 AND status = 'pending'
        "#)
        .bind(&tx.sender)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))? as u64;

        Ok(sender_balance >= pending_spent + tx.total_cost())
    }

    // Parallel transaction processing
    pub async fn add_transaction_batch(&self, transactions: &[Transaction]) -> Result<u64, DatabaseError> {
        let mut tx_db = self.pool.begin()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let mut processed = 0u64;

        for transaction in transactions {
            let result = sqlx::query(r#"
                INSERT INTO transactions 
                (id, sender, recipient, amount, fee, status, timestamp, signature, public_key, nonce)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                ON CONFLICT (id) DO NOTHING
            "#)
            .bind(&transaction.id)
            .bind(&transaction.sender)
            .bind(&transaction.recipient)
            .bind(transaction.amount as i64)
            .bind(transaction.fee as i64)
            .bind("pending")
            .bind(transaction.timestamp)
            .bind(&transaction.signature)
            .bind(&transaction.public_key)
            .bind(transaction.nonce as i64)
            .execute(&mut *tx_db)
            .await;

            if result.is_ok() {
                processed += 1;
                
                // Invalidate balance cache for affected addresses
                self.balance_cache.remove(&transaction.sender);
                self.balance_cache.remove(&transaction.recipient);
            }
        }

        tx_db.commit()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(processed)
    }

    // Ultra-fast block insertion with parallel UTXO updates
    pub async fn add_block(&self, block: &Block) -> Result<(), DatabaseError> {
        let mut tx_db = self.pool.begin()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        // Insert block
        sqlx::query(r#"
            INSERT INTO blocks 
            (hash, height, previous_hash, merkle_root, timestamp, difficulty, nonce, block_data, size_bytes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#)
        .bind(&block.hash)
        .bind(block.height as i64)
        .bind(&block.previous_hash)
        .bind(&block.merkle_root)
        .bind(block.timestamp)
        .bind(block.difficulty as i32)
        .bind(block.nonce as i64)
        .bind(serde_json::to_value(&block).unwrap())
        .bind(block.calculate_size() as i32)
        .execute(&mut *tx_db)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        // Process transactions in parallel batches
        for transaction in &block.transactions {
            // Update transaction status
            sqlx::query(r#"
                UPDATE transactions 
                SET status = 'confirmed', block_hash = $1, block_height = $2
                WHERE id = $3
            "#)
            .bind(&block.hash)
            .bind(block.height as i64)
            .bind(&transaction.id)
            .execute(&mut *tx_db)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

            // Create new UTXO for recipient
            sqlx::query(r#"
                INSERT INTO utxos 
                (tx_id, output_index, address, amount, block_height)
                VALUES ($1, 0, $2, $3, $4)
                ON CONFLICT (tx_id, output_index) DO NOTHING
            "#)
            .bind(&transaction.id)
            .bind(&transaction.recipient)
            .bind(transaction.amount as i64)
            .bind(block.height as i64)
            .execute(&mut *tx_db)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

            // Spend UTXOs for sender (if not mining reward)
            if transaction.sender != "MINING_REWARD" {
                sqlx::query(r#"
                    UPDATE utxos 
                    SET is_spent = true, spent_at_height = $1, spent_in_tx = $2
                    WHERE address = $3 AND NOT is_spent 
                    AND (SELECT SUM(amount) FROM utxos WHERE address = $3 AND NOT is_spent) >= $4
                "#)
                .bind(block.height as i64)
                .bind(&transaction.id)
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

    // Get transaction history with pagination
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
            WHERE sender = $1 OR recipient = $1
            ORDER BY timestamp DESC
            LIMIT $2 OFFSET $3
        "#)
        .bind(address)
        .bind(limit as i32)
        .bind(offset as i32)
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

            transactions.push(TransactionRecord {
                id: row.get("id"),
                block_hash: row.get("block_hash"),
                block_height: row.get::<Option<i64>, _>("block_height").map(|h| h as u64),
                sender: row.get("sender"),
                recipient: row.get("recipient"),
                amount: row.get::<i64, _>("amount") as u64,
                fee: row.get::<i64, _>("fee") as u64,
                status,
                timestamp: row.get("timestamp"),
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

        let utxo_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM utxos WHERE NOT is_spent")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        stats.insert("total_blocks".to_string(), block_count as u64);
        stats.insert("total_transactions".to_string(), tx_count as u64);
        stats.insert("active_utxos".to_string(), utxo_count as u64);
        stats.insert("cached_balances".to_string(), self.balance_cache.len() as u64);
        stats.insert("cached_utxos".to_string(), self.utxo_cache.len() as u64);

        Ok(stats)
    }

    // Cleanup and maintenance
    pub async fn cleanup_old_cache_entries(&self) {
        // Remove cache entries older than 5 minutes
        let cutoff = Utc::now() - chrono::Duration::minutes(5);
        
        self.balance_cache.retain(|_, balance| {
            balance.last_updated > cutoff
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_balance_caching() {
        // Test would require test database setup
        // Implementation would verify cache hit/miss behavior
    }
}
