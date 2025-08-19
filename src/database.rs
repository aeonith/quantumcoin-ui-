use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use sqlx::{SqlitePool, Row, sqlite::SqliteConnectOptions};
use std::path::Path;
use tokio::sync::RwLock;
use std::sync::Arc;
use chrono::{DateTime, Utc};

use crate::{
    block::Block,
    transaction::{Transaction, SignedTransaction},
    utxo::{UTXO, UTXOSet},
};

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub database_path: String,
    pub max_connections: u32,
    pub auto_vacuum: bool,
    pub journal_mode: JournalMode,
    pub synchronous: SynchronousMode,
    pub cache_size: i32,
}

#[derive(Debug, Clone)]
pub enum JournalMode {
    Delete,
    Truncate,
    Persist,
    Memory,
    WAL,
    Off,
}

impl JournalMode {
    pub fn as_str(&self) -> &str {
        match self {
            JournalMode::Delete => "DELETE",
            JournalMode::Truncate => "TRUNCATE", 
            JournalMode::Persist => "PERSIST",
            JournalMode::Memory => "MEMORY",
            JournalMode::WAL => "WAL",
            JournalMode::Off => "OFF",
        }
    }
}

#[derive(Debug, Clone)]
pub enum SynchronousMode {
    Off,
    Normal,
    Full,
    Extra,
}

impl SynchronousMode {
    pub fn as_str(&self) -> &str {
        match self {
            SynchronousMode::Off => "OFF",
            SynchronousMode::Normal => "NORMAL",
            SynchronousMode::Full => "FULL",
            SynchronousMode::Extra => "EXTRA",
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_path: "quantumcoin.db".to_string(),
            max_connections: 10,
            auto_vacuum: true,
            journal_mode: JournalMode::WAL, // Write-Ahead Logging for better concurrency
            synchronous: SynchronousMode::Full, // Full durability
            cache_size: -64000, // 64MB cache (negative means KB)
        }
    }
}

/// Block storage entry
#[derive(Debug, Serialize, Deserialize)]
struct BlockEntry {
    pub height: u64,
    pub hash: String,
    pub previous_hash: String,
    pub merkle_root: String,
    pub timestamp: DateTime<Utc>,
    pub difficulty: u32,
    pub nonce: u64,
    pub transaction_count: u32,
    pub block_size: u64,
    pub data: Vec<u8>, // Serialized block data
}

/// Transaction storage entry
#[derive(Debug, Serialize, Deserialize)]
struct TransactionEntry {
    pub txid: String,
    pub block_hash: String,
    pub block_height: u64,
    pub transaction_index: u32,
    pub version: u32,
    pub lock_time: u32,
    pub input_count: u32,
    pub output_count: u32,
    pub fee: u64,
    pub size: u32,
    pub timestamp: DateTime<Utc>,
    pub data: Vec<u8>, // Serialized transaction data
}

/// UTXO storage entry
#[derive(Debug, Serialize, Deserialize)]
struct UTXOEntry {
    pub outpoint: String, // txid:output_index
    pub txid: String,
    pub output_index: u32,
    pub amount: u64,
    pub address: String,
    pub script_pubkey: Vec<u8>,
    pub block_height: u64,
    pub is_coinbase: bool,
    pub spent_in_tx: Option<String>, // NULL if unspent
    pub spent_at_height: Option<u64>,
}

/// Database-backed blockchain storage
pub struct BlockchainDatabase {
    pool: SqlitePool,
    config: DatabaseConfig,
    utxo_cache: Arc<RwLock<UTXOSet>>,
    write_buffer: Arc<RwLock<WriteBuffer>>,
}

/// Write buffer for batching database operations
#[derive(Debug, Default)]
struct WriteBuffer {
    blocks: Vec<BlockEntry>,
    transactions: Vec<TransactionEntry>,
    utxos: Vec<UTXOEntry>,
    spent_utxos: Vec<String>, // Outpoints of spent UTXOs
}

impl BlockchainDatabase {
    /// Create new database connection
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = Path::new(&config.database_path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Create connection options
        let options = SqliteConnectOptions::new()
            .filename(&config.database_path)
            .create_if_missing(true)
            .journal_mode(match config.journal_mode {
                JournalMode::Delete => sqlx::sqlite::SqliteJournalMode::Delete,
                JournalMode::Truncate => sqlx::sqlite::SqliteJournalMode::Truncate,
                JournalMode::Persist => sqlx::sqlite::SqliteJournalMode::Persist,
                JournalMode::Memory => sqlx::sqlite::SqliteJournalMode::Memory,
                JournalMode::WAL => sqlx::sqlite::SqliteJournalMode::Wal,
                JournalMode::Off => sqlx::sqlite::SqliteJournalMode::Off,
            })
            .synchronous(match config.synchronous {
                SynchronousMode::Off => sqlx::sqlite::SqliteSynchronous::Off,
                SynchronousMode::Normal => sqlx::sqlite::SqliteSynchronous::Normal,
                SynchronousMode::Full => sqlx::sqlite::SqliteSynchronous::Full,
                SynchronousMode::Extra => sqlx::sqlite::SqliteSynchronous::Extra,
            })
            .auto_vacuum(match config.auto_vacuum {
                true => sqlx::sqlite::SqliteAutoVacuum::Full,
                false => sqlx::sqlite::SqliteAutoVacuum::None,
            });

        // Create connection pool
        let pool = sqlx::SqlitePool::connect_with(options).await
            .context("Failed to connect to database")?;

        // Set cache size
        sqlx::query(&format!("PRAGMA cache_size = {}", config.cache_size))
            .execute(&pool)
            .await?;

        let mut db = Self {
            pool,
            config,
            utxo_cache: Arc::new(RwLock::new(UTXOSet::new())),
            write_buffer: Arc::new(RwLock::new(WriteBuffer::default())),
        };

        // Initialize database schema
        db.initialize_schema().await?;

        // Load UTXO set into memory
        db.load_utxo_cache().await?;

        Ok(db)
    }

    /// Initialize database schema
    async fn initialize_schema(&self) -> Result<()> {
        // Create blocks table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS blocks (
                height INTEGER PRIMARY KEY,
                hash TEXT UNIQUE NOT NULL,
                previous_hash TEXT NOT NULL,
                merkle_root TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                difficulty INTEGER NOT NULL,
                nonce INTEGER NOT NULL,
                transaction_count INTEGER NOT NULL,
                block_size INTEGER NOT NULL,
                data BLOB NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
        "#).execute(&self.pool).await?;

        // Create transactions table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS transactions (
                txid TEXT PRIMARY KEY,
                block_hash TEXT NOT NULL,
                block_height INTEGER NOT NULL,
                transaction_index INTEGER NOT NULL,
                version INTEGER NOT NULL,
                lock_time INTEGER NOT NULL,
                input_count INTEGER NOT NULL,
                output_count INTEGER NOT NULL,
                fee INTEGER NOT NULL,
                size INTEGER NOT NULL,
                timestamp TEXT NOT NULL,
                data BLOB NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (block_hash) REFERENCES blocks(hash) ON DELETE CASCADE
            )
        "#).execute(&self.pool).await?;

        // Create UTXOs table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS utxos (
                outpoint TEXT PRIMARY KEY,
                txid TEXT NOT NULL,
                output_index INTEGER NOT NULL,
                amount INTEGER NOT NULL,
                address TEXT NOT NULL,
                script_pubkey BLOB NOT NULL,
                block_height INTEGER NOT NULL,
                is_coinbase BOOLEAN NOT NULL,
                spent_in_tx TEXT NULL,
                spent_at_height INTEGER NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
        "#).execute(&self.pool).await?;

        // Create indices for performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_blocks_hash ON blocks(hash)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_blocks_height ON blocks(height)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_transactions_block ON transactions(block_hash)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_transactions_height ON transactions(block_height)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_utxos_address ON utxos(address)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_utxos_spent ON utxos(spent_in_tx) WHERE spent_in_tx IS NULL").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_utxos_height ON utxos(block_height)").execute(&self.pool).await?;

        // Create chain state table for metadata
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS chain_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
        "#).execute(&self.pool).await?;

        Ok(())
    }

    /// Store a block in the database
    pub async fn store_block(&self, block: &Block, transactions: &[SignedTransaction]) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Serialize block data
        let block_data = bincode::serialize(block)?;
        let block_size = block_data.len() as u64;

        // Insert block
        sqlx::query(r#"
            INSERT INTO blocks (height, hash, previous_hash, merkle_root, timestamp, difficulty, nonce, transaction_count, block_size, data)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(block.index as i64)
        .bind(&block.hash)
        .bind(&block.previous_hash)
        .bind(&block.merkle_root)
        .bind(block.timestamp.to_rfc3339())
        .bind(block.difficulty as i64)
        .bind(block.nonce as i64)
        .bind(transactions.len() as i64)
        .bind(block_size as i64)
        .bind(block_data)
        .execute(&mut *tx).await?;

        // Insert transactions
        for (index, transaction) in transactions.iter().enumerate() {
            let tx_data = bincode::serialize(transaction)?;
            let tx_size = tx_data.len() as u32;

            sqlx::query(r#"
                INSERT INTO transactions (txid, block_hash, block_height, transaction_index, version, lock_time, input_count, output_count, fee, size, timestamp, data)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#)
            .bind(&transaction.id)
            .bind(&block.hash)
            .bind(block.index as i64)
            .bind(index as i64)
            .bind(transaction.version as i64)
            .bind(transaction.lock_time as i64)
            .bind(transaction.inputs.len() as i64)
            .bind(transaction.outputs.len() as i64)
            .bind(transaction.calculate_fee(&std::collections::HashMap::new()).unwrap_or(0) as i64)
            .bind(tx_size as i64)
            .bind(transaction.timestamp.to_rfc3339())
            .bind(tx_data)
            .execute(&mut *tx).await?;

            // Update UTXO set
            self.update_utxos_for_transaction(transaction, block.index, &mut tx).await?;
        }

        // Update chain state
        self.update_chain_state("best_block_hash", &block.hash, &mut tx).await?;
        self.update_chain_state("best_block_height", &block.index.to_string(), &mut tx).await?;

        // Commit transaction
        tx.commit().await?;

        // Update in-memory UTXO cache
        {
            let mut utxo_cache = self.utxo_cache.write().await;
            for transaction in transactions {
                let is_coinbase = transaction.inputs.len() == 1 && transaction.inputs[0].previous_output.starts_with("coinbase");
                utxo_cache.apply_transaction(transaction, block.index, is_coinbase)?;
            }
            utxo_cache.set_height(block.index);
        }

        Ok(())
    }

    /// Update UTXOs for a transaction
    async fn update_utxos_for_transaction(
        &self,
        transaction: &SignedTransaction,
        block_height: u64,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<()> {
        let is_coinbase = transaction.inputs.len() == 1 && transaction.inputs[0].previous_output.starts_with("coinbase");

        // Spend inputs (mark UTXOs as spent)
        if !is_coinbase {
            for input in &transaction.inputs {
                sqlx::query(r#"
                    UPDATE utxos 
                    SET spent_in_tx = ?, spent_at_height = ?
                    WHERE outpoint = ?
                "#)
                .bind(&transaction.id)
                .bind(block_height as i64)
                .bind(&input.previous_output)
                .execute(&mut **tx).await?;
            }
        }

        // Create new UTXOs from outputs
        for (output_index, output) in transaction.outputs.iter().enumerate() {
            let outpoint = format!("{}:{}", transaction.id, output_index);
            
            sqlx::query(r#"
                INSERT INTO utxos (outpoint, txid, output_index, amount, address, script_pubkey, block_height, is_coinbase)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#)
            .bind(&outpoint)
            .bind(&transaction.id)
            .bind(output_index as i64)
            .bind(output.value as i64)
            .bind(&output.address)
            .bind(&output.script_pubkey)
            .bind(block_height as i64)
            .bind(is_coinbase)
            .execute(&mut **tx).await?;
        }

        Ok(())
    }

    /// Get block by height
    pub async fn get_block_by_height(&self, height: u64) -> Result<Option<Block>> {
        let row = sqlx::query("SELECT data FROM blocks WHERE height = ?")
            .bind(height as i64)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let data: Vec<u8> = row.get("data");
            let block: Block = bincode::deserialize(&data)?;
            Ok(Some(block))
        } else {
            Ok(None)
        }
    }

    /// Get block by hash
    pub async fn get_block_by_hash(&self, hash: &str) -> Result<Option<Block>> {
        let row = sqlx::query("SELECT data FROM blocks WHERE hash = ?")
            .bind(hash)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let data: Vec<u8> = row.get("data");
            let block: Block = bincode::deserialize(&data)?;
            Ok(Some(block))
        } else {
            Ok(None)
        }
    }

    /// Get transaction by ID
    pub async fn get_transaction(&self, txid: &str) -> Result<Option<SignedTransaction>> {
        let row = sqlx::query("SELECT data FROM transactions WHERE txid = ?")
            .bind(txid)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let data: Vec<u8> = row.get("data");
            let tx: SignedTransaction = bincode::deserialize(&data)?;
            Ok(Some(tx))
        } else {
            Ok(None)
        }
    }

    /// Get current blockchain height
    pub async fn get_chain_height(&self) -> Result<u64> {
        let row = sqlx::query("SELECT value FROM chain_state WHERE key = 'best_block_height'")
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let height_str: String = row.get("value");
            Ok(height_str.parse().unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    /// Get UTXO set (from cache)
    pub async fn get_utxo_set(&self) -> UTXOSet {
        self.utxo_cache.read().await.clone()
    }

    /// Get balance for an address
    pub async fn get_balance(&self, address: &str) -> Result<u64> {
        let row = sqlx::query(r#"
            SELECT COALESCE(SUM(amount), 0) as balance 
            FROM utxos 
            WHERE address = ? AND spent_in_tx IS NULL
        "#)
        .bind(address)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get::<i64, _>("balance") as u64)
    }

    /// Get UTXOs for an address
    pub async fn get_utxos_for_address(&self, address: &str) -> Result<Vec<UTXO>> {
        let rows = sqlx::query(r#"
            SELECT outpoint, txid, output_index, amount, address, script_pubkey, block_height, is_coinbase
            FROM utxos 
            WHERE address = ? AND spent_in_tx IS NULL
        "#)
        .bind(address)
        .fetch_all(&self.pool)
        .await?;

        let mut utxos = Vec::new();
        for row in rows {
            let utxo = UTXO {
                tx_id: row.get("txid"),
                output_index: row.get::<i64, _>("output_index") as u32,
                amount: row.get::<i64, _>("amount") as u64,
                script_pubkey: row.get("script_pubkey"),
                address: row.get("address"),
                block_height: row.get::<i64, _>("block_height") as u64,
                is_coinbase: row.get("is_coinbase"),
                confirmations: 0, // Will be calculated by caller
            };
            utxos.push(utxo);
        }

        Ok(utxos)
    }

    /// Load UTXO cache from database
    async fn load_utxo_cache(&self) -> Result<()> {
        let rows = sqlx::query(r#"
            SELECT outpoint, txid, output_index, amount, address, script_pubkey, block_height, is_coinbase
            FROM utxos 
            WHERE spent_in_tx IS NULL
        "#)
        .fetch_all(&self.pool)
        .await?;

        let mut utxo_cache = self.utxo_cache.write().await;

        for row in rows {
            let utxo = UTXO {
                tx_id: row.get("txid"),
                output_index: row.get::<i64, _>("output_index") as u32,
                amount: row.get::<i64, _>("amount") as u64,
                script_pubkey: row.get("script_pubkey"),
                address: row.get("address"),
                block_height: row.get::<i64, _>("block_height") as u64,
                is_coinbase: row.get("is_coinbase"),
                confirmations: 0,
            };

            utxo_cache.add_utxo(utxo)?;
        }

        // Set current height
        let height = self.get_chain_height().await?;
        utxo_cache.set_height(height);

        Ok(())
    }

    /// Update chain state
    async fn update_chain_state(
        &self,
        key: &str,
        value: &str,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<()> {
        sqlx::query(r#"
            INSERT INTO chain_state (key, value, updated_at) 
            VALUES (?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(key) DO UPDATE SET 
                value = excluded.value,
                updated_at = excluded.updated_at
        "#)
        .bind(key)
        .bind(value)
        .execute(&mut **tx).await?;

        Ok(())
    }

    /// Perform database maintenance
    pub async fn maintenance(&self) -> Result<()> {
        // Vacuum database to reclaim space
        sqlx::query("VACUUM").execute(&self.pool).await?;

        // Analyze tables for better query planning
        sqlx::query("ANALYZE").execute(&self.pool).await?;

        // Integrity check
        let row = sqlx::query("PRAGMA integrity_check")
            .fetch_one(&self.pool)
            .await?;

        let integrity_result: String = row.get(0);
        if integrity_result != "ok" {
            anyhow::bail!("Database integrity check failed: {}", integrity_result);
        }

        Ok(())
    }

    /// Close database connection
    pub async fn close(self) {
        self.pool.close().await;
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        let block_count = sqlx::query("SELECT COUNT(*) as count FROM blocks")
            .fetch_one(&self.pool).await?
            .get::<i64, _>("count") as u64;

        let tx_count = sqlx::query("SELECT COUNT(*) as count FROM transactions")
            .fetch_one(&self.pool).await?
            .get::<i64, _>("count") as u64;

        let utxo_count = sqlx::query("SELECT COUNT(*) as count FROM utxos WHERE spent_in_tx IS NULL")
            .fetch_one(&self.pool).await?
            .get::<i64, _>("count") as u64;

        let total_value = sqlx::query("SELECT COALESCE(SUM(amount), 0) as total FROM utxos WHERE spent_in_tx IS NULL")
            .fetch_one(&self.pool).await?
            .get::<i64, _>("total") as u64;

        // Get database size
        let db_size = tokio::fs::metadata(&self.config.database_path).await
            .map(|meta| meta.len())
            .unwrap_or(0);

        Ok(DatabaseStats {
            block_count,
            transaction_count: tx_count,
            utxo_count,
            total_value,
            database_size: db_size,
        })
    }
}

/// Database statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub block_count: u64,
    pub transaction_count: u64,
    pub utxo_count: u64,
    pub total_value: u64,
    pub database_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::transaction::{TransactionInput, TransactionOutput};

    async fn create_test_db() -> Result<BlockchainDatabase> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test.db");
        
        let config = DatabaseConfig {
            database_path: db_path.to_string_lossy().to_string(),
            ..DatabaseConfig::default()
        };

        BlockchainDatabase::new(config).await
    }

    #[tokio::test]
    async fn test_database_initialization() -> Result<()> {
        let db = create_test_db().await?;
        
        // Check that tables were created
        let height = db.get_chain_height().await?;
        assert_eq!(height, 0);

        let stats = db.get_stats().await?;
        assert_eq!(stats.block_count, 0);
        assert_eq!(stats.transaction_count, 0);
        assert_eq!(stats.utxo_count, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_block_storage_retrieval() -> Result<()> {
        let db = create_test_db().await?;
        
        // Create test block
        let block = Block::new(
            1,
            "0000000000000000".to_string(),
            vec![], // Empty for test
            4,
        );

        let transactions = vec![];
        
        // Store block
        db.store_block(&block, &transactions).await?;

        // Retrieve block by height
        let retrieved_block = db.get_block_by_height(1).await?;
        assert!(retrieved_block.is_some());
        
        let retrieved = retrieved_block.unwrap();
        assert_eq!(retrieved.index, block.index);
        assert_eq!(retrieved.hash, block.hash);

        // Retrieve block by hash
        let by_hash = db.get_block_by_hash(&block.hash).await?;
        assert!(by_hash.is_some());

        // Check chain height updated
        let height = db.get_chain_height().await?;
        assert_eq!(height, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_utxo_management() -> Result<()> {
        let db = create_test_db().await?;

        // Create transaction with outputs
        let tx = SignedTransaction {
            id: "test_tx_1".to_string(),
            version: 1,
            inputs: vec![
                TransactionInput {
                    previous_output: "coinbase:0".to_string(),
                    script_sig: vec![],
                    sequence: 0xffffffff,
                }
            ],
            outputs: vec![
                TransactionOutput {
                    value: 5000000000, // 50 QTC
                    script_pubkey: vec![],
                    address: "alice".to_string(),
                },
                TransactionOutput {
                    value: 2500000000, // 25 QTC
                    script_pubkey: vec![],
                    address: "bob".to_string(),
                }
            ],
            lock_time: 0,
            timestamp: Utc::now(),
            signature: "test_sig".to_string(),
            public_key: "test_pub".to_string(),
        };

        let block = Block::new(1, "genesis".to_string(), vec![], 4);

        // Store block with transaction
        db.store_block(&block, &[tx.clone()]).await?;

        // Check balances
        let alice_balance = db.get_balance("alice").await?;
        let bob_balance = db.get_balance("bob").await?;
        
        assert_eq!(alice_balance, 5000000000);
        assert_eq!(bob_balance, 2500000000);

        // Check UTXOs
        let alice_utxos = db.get_utxos_for_address("alice").await?;
        let bob_utxos = db.get_utxos_for_address("bob").await?;
        
        assert_eq!(alice_utxos.len(), 1);
        assert_eq!(bob_utxos.len(), 1);
        assert_eq!(alice_utxos[0].amount, 5000000000);
        assert_eq!(bob_utxos[0].amount, 2500000000);

        Ok(())
    }

    #[tokio::test]
    async fn test_database_stats() -> Result<()> {
        let db = create_test_db().await?;

        // Add some test data
        let tx = SignedTransaction {
            id: "test_tx".to_string(),
            version: 1,
            inputs: vec![
                TransactionInput {
                    previous_output: "coinbase:0".to_string(),
                    script_sig: vec![],
                    sequence: 0xffffffff,
                }
            ],
            outputs: vec![
                TransactionOutput {
                    value: 1000000000,
                    script_pubkey: vec![],
                    address: "test".to_string(),
                }
            ],
            lock_time: 0,
            timestamp: Utc::now(),
            signature: "sig".to_string(),
            public_key: "pub".to_string(),
        };

        let block = Block::new(1, "genesis".to_string(), vec![], 4);
        db.store_block(&block, &[tx]).await?;

        // Check stats
        let stats = db.get_stats().await?;
        
        assert_eq!(stats.block_count, 1);
        assert_eq!(stats.transaction_count, 1);
        assert_eq!(stats.utxo_count, 1);
        assert_eq!(stats.total_value, 1000000000);
        assert!(stats.database_size > 0);

        Ok(())
    }
}
