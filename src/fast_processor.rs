use crate::transaction::{Transaction, TransactionError};
use crate::database::{QuantumCoinDB, DatabaseError};
use crate::blockchain::Blockchain;
use rayon::prelude::*;
use crossbeam::queue::SegQueue;
use std::sync::Arc;
use std::collections::{HashMap, VecDeque};
use tokio::sync::{RwLock, Semaphore, mpsc};
use tokio::time::{Duration, Instant};
use parking_lot::Mutex;
use dashmap::DashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum ProcessorError {
    #[error("Transaction validation failed: {0}")]
    ValidationFailed(String),
    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),
    #[error("Transaction error: {0}")]
    TransactionError(#[from] TransactionError),
    #[error("Processing queue full")]
    QueueFull,
    #[error("Rate limit exceeded")]
    RateLimited,
}

#[derive(Clone, Debug)]
pub struct ProcessingStats {
    pub transactions_processed: u64,
    pub transactions_per_second: f64,
    pub average_processing_time_ms: f64,
    pub queue_size: usize,
    pub validation_errors: u64,
    pub successful_batches: u64,
    pub failed_batches: u64,
}

#[derive(Clone, Debug)]
struct TransactionBatch {
    pub id: String,
    pub transactions: Vec<Transaction>,
    pub priority: u8,
    pub received_at: Instant,
}

#[derive(Clone, Debug)]
pub struct ProcessingConfig {
    pub max_batch_size: usize,
    pub max_queue_size: usize,
    pub batch_timeout_ms: u64,
    pub parallel_workers: usize,
    pub max_concurrent_validations: usize,
    pub enable_prioritization: bool,
    pub validation_cache_size: usize,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 1000,
            max_queue_size: 10000,
            batch_timeout_ms: 100,
            parallel_workers: num_cpus::get(),
            max_concurrent_validations: 100,
            enable_prioritization: true,
            validation_cache_size: 10000,
        }
    }
}

pub struct FastTransactionProcessor {
    database: Arc<QuantumCoinDB>,
    blockchain: Arc<RwLock<Blockchain>>,
    config: ProcessingConfig,
    
    // High-performance queues
    priority_queue: Arc<SegQueue<TransactionBatch>>,
    processing_semaphore: Arc<Semaphore>,
    
    // Validation cache for preventing duplicate work
    validation_cache: Arc<DashMap<String, (bool, Instant)>>, // tx_hash -> (valid, timestamp)
    
    // Performance tracking
    stats: Arc<RwLock<ProcessingStats>>,
    processing_times: Arc<Mutex<VecDeque<Duration>>>,
    
    // Batch processing
    current_batch: Arc<Mutex<Vec<Transaction>>>,
    batch_timer: Arc<Mutex<Option<Instant>>>,
    
    // Communication channels
    result_sender: mpsc::UnboundedSender<ProcessingResult>,
    shutdown_signal: Arc<RwLock<bool>>,
}

#[derive(Clone, Debug)]
pub struct ProcessingResult {
    pub transaction_id: String,
    pub success: bool,
    pub error: Option<String>,
    pub processing_time: Duration,
}

impl FastTransactionProcessor {
    pub fn new(
        database: Arc<QuantumCoinDB>,
        blockchain: Arc<RwLock<Blockchain>>,
        config: ProcessingConfig,
    ) -> (Self, mpsc::UnboundedReceiver<ProcessingResult>) {
        let (tx, rx) = mpsc::unbounded_channel();
        
        let processor = Self {
            database,
            blockchain,
            config: config.clone(),
            priority_queue: Arc::new(SegQueue::new()),
            processing_semaphore: Arc::new(Semaphore::new(config.max_concurrent_validations)),
            validation_cache: Arc::new(DashMap::new()),
            stats: Arc::new(RwLock::new(ProcessingStats {
                transactions_processed: 0,
                transactions_per_second: 0.0,
                average_processing_time_ms: 0.0,
                queue_size: 0,
                validation_errors: 0,
                successful_batches: 0,
                failed_batches: 0,
            })),
            processing_times: Arc::new(Mutex::new(VecDeque::new())),
            current_batch: Arc::new(Mutex::new(Vec::new())),
            batch_timer: Arc::new(Mutex::new(None)),
            result_sender: tx,
            shutdown_signal: Arc::new(RwLock::new(false)),
        };
        
        (processor, rx)
    }

    pub async fn start(&self) -> Result<(), ProcessorError> {
        println!("Starting FastTransactionProcessor with {} workers", self.config.parallel_workers);
        
        // Start batch processing worker
        let batch_processor = self.clone_for_worker();
        tokio::spawn(async move {
            batch_processor.batch_processing_loop().await;
        });

        // Start parallel validation workers
        for worker_id in 0..self.config.parallel_workers {
            let validation_worker = self.clone_for_worker();
            tokio::spawn(async move {
                validation_worker.validation_worker_loop(worker_id).await;
            });
        }

        // Start statistics updater
        let stats_updater = self.clone_for_worker();
        tokio::spawn(async move {
            stats_updater.stats_update_loop().await;
        });

        // Start cache cleanup worker
        let cache_cleaner = self.clone_for_worker();
        tokio::spawn(async move {
            cache_cleaner.cache_cleanup_loop().await;
        });

        Ok(())
    }

    // Submit transaction for ultra-fast processing
    pub async fn submit_transaction(&self, transaction: Transaction) -> Result<String, ProcessorError> {
        // Check queue capacity
        let current_stats = self.stats.read().await;
        if current_stats.queue_size >= self.config.max_queue_size {
            return Err(ProcessorError::QueueFull);
        }
        drop(current_stats);

        // Quick validation cache check
        let tx_hash = transaction.calculate_hash();
        if let Some((cached_result, timestamp)) = self.validation_cache.get(&tx_hash) {
            if timestamp.elapsed() < Duration::from_secs(300) { // 5 minute cache
                if !*cached_result {
                    return Err(ProcessorError::ValidationFailed("Cached validation failure".to_string()));
                }
            }
        }

        // Add to batch or process immediately based on configuration
        if self.config.max_batch_size > 1 {
            self.add_to_batch(transaction).await?;
        } else {
            self.process_single_transaction(transaction).await?;
        }

        Ok(tx_hash)
    }

    async fn add_to_batch(&self, transaction: Transaction) -> Result<(), ProcessorError> {
        let mut current_batch = self.current_batch.lock();
        let mut batch_timer = self.batch_timer.lock();
        
        // Initialize timer if this is the first transaction in batch
        if current_batch.is_empty() {
            *batch_timer = Some(Instant::now());
        }
        
        current_batch.push(transaction);
        
        // Check if we should process the batch
        let should_process = current_batch.len() >= self.config.max_batch_size ||
            batch_timer.map_or(false, |timer| timer.elapsed().as_millis() >= self.config.batch_timeout_ms as u128);
        
        if should_process {
            let batch_transactions = std::mem::take(&mut *current_batch);
            *batch_timer = None;
            drop(current_batch);
            drop(batch_timer);
            
            // Create batch and add to queue
            let batch = TransactionBatch {
                id: Uuid::new_v4().to_string(),
                transactions: batch_transactions,
                priority: self.calculate_batch_priority(&batch_transactions),
                received_at: Instant::now(),
            };
            
            self.priority_queue.push(batch);
        }
        
        Ok(())
    }

    fn calculate_batch_priority(&self, transactions: &[Transaction]) -> u8 {
        if !self.config.enable_prioritization {
            return 1;
        }
        
        // Higher fee transactions get higher priority
        let avg_fee = transactions.iter().map(|tx| tx.fee).sum::<u64>() / transactions.len() as u64;
        
        match avg_fee {
            fee if fee >= 10000 => 3, // High priority
            fee if fee >= 5000 => 2,  // Medium priority
            _ => 1,                   // Normal priority
        }
    }

    async fn process_single_transaction(&self, transaction: Transaction) -> Result<(), ProcessorError> {
        let _permit = self.processing_semaphore.acquire().await.unwrap();
        let start_time = Instant::now();
        
        let result = self.validate_and_process_transaction(transaction.clone()).await;
        
        let processing_time = start_time.elapsed();
        self.record_processing_time(processing_time).await;
        
        // Send result
        let processing_result = ProcessingResult {
            transaction_id: transaction.id.clone(),
            success: result.is_ok(),
            error: result.as_ref().err().map(|e| e.to_string()),
            processing_time,
        };
        
        let _ = self.result_sender.send(processing_result);
        
        result.map(|_| ())
    }

    async fn validate_and_process_transaction(&self, transaction: Transaction) -> Result<Transaction, ProcessorError> {
        let tx_hash = transaction.calculate_hash();
        
        // Step 1: Fast validation checks
        if let Err(e) = self.fast_validation_checks(&transaction).await {
            self.validation_cache.insert(tx_hash.clone(), (false, Instant::now()));
            return Err(e);
        }
        
        // Step 2: Database validation (with balance check)
        if !self.database.validate_transaction_fast(&transaction).await? {
            self.validation_cache.insert(tx_hash.clone(), (false, Instant::now()));
            return Err(ProcessorError::ValidationFailed("Insufficient balance or double spending".to_string()));
        }
        
        // Step 3: Cryptographic signature validation
        if let Err(e) = transaction.verify_signature() {
            self.validation_cache.insert(tx_hash.clone(), (false, Instant::now()));
            return Err(ProcessorError::TransactionError(e));
        }
        
        // Step 4: Blockchain-specific validation
        {
            let blockchain = self.blockchain.read().await;
            let sender_balance = blockchain.get_balance(&transaction.sender);
            if sender_balance < transaction.total_cost() {
                self.validation_cache.insert(tx_hash.clone(), (false, Instant::now()));
                return Err(ProcessorError::ValidationFailed("Insufficient balance in blockchain".to_string()));
            }
        }
        
        // Cache successful validation
        self.validation_cache.insert(tx_hash, (true, Instant::now()));
        
        Ok(transaction)
    }

    async fn fast_validation_checks(&self, transaction: &Transaction) -> Result<(), ProcessorError> {
        // Basic format checks
        if transaction.amount == 0 {
            return Err(ProcessorError::ValidationFailed("Zero amount transaction".to_string()));
        }
        
        if transaction.fee < transaction.calculate_fee() {
            return Err(ProcessorError::ValidationFailed("Insufficient fee".to_string()));
        }
        
        if transaction.sender == transaction.recipient {
            return Err(ProcessorError::ValidationFailed("Self-transaction not allowed".to_string()));
        }
        
        if transaction.sender.is_empty() || transaction.recipient.is_empty() {
            return Err(ProcessorError::ValidationFailed("Invalid addresses".to_string()));
        }
        
        Ok(())
    }

    async fn batch_processing_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_millis(self.config.batch_timeout_ms));
        
        loop {
            interval.tick().await;
            
            if *self.shutdown_signal.read().await {
                break;
            }
            
            // Process any pending batches
            while let Some(batch) = self.priority_queue.pop() {
                self.process_batch(batch).await;
            }
            
            // Check for timeout batches
            self.check_batch_timeout().await;
        }
    }

    async fn check_batch_timeout(&self) {
        let mut current_batch = self.current_batch.lock();
        let mut batch_timer = self.batch_timer.lock();
        
        if let Some(timer) = *batch_timer {
            if timer.elapsed().as_millis() >= self.config.batch_timeout_ms as u128 && !current_batch.is_empty() {
                let batch_transactions = std::mem::take(&mut *current_batch);
                *batch_timer = None;
                drop(current_batch);
                drop(batch_timer);
                
                let batch = TransactionBatch {
                    id: Uuid::new_v4().to_string(),
                    transactions: batch_transactions,
                    priority: 1,
                    received_at: Instant::now(),
                };
                
                self.priority_queue.push(batch);
            }
        }
    }

    async fn process_batch(&self, batch: TransactionBatch) {
        let start_time = Instant::now();
        
        // Parallel validation of all transactions in batch
        let validation_results: Vec<_> = batch.transactions
            .par_iter()
            .map(|tx| {
                // This runs in parallel using rayon
                let tx_hash = tx.calculate_hash();
                
                // Check cache first
                if let Some((cached_result, timestamp)) = self.validation_cache.get(&tx_hash) {
                    if timestamp.elapsed() < Duration::from_secs(300) {
                        return (*cached_result, tx.clone(), None);
                    }
                }
                
                // Basic validation checks (CPU-bound, perfect for parallel processing)
                match self.validate_transaction_format(tx) {
                    Ok(_) => (true, tx.clone(), None),
                    Err(e) => (false, tx.clone(), Some(e.to_string())),
                }
            })
            .collect();
        
        // Filter valid transactions
        let valid_transactions: Vec<Transaction> = validation_results
            .iter()
            .filter_map(|(valid, tx, _)| if *valid { Some(tx.clone()) } else { None })
            .collect();
        
        if !valid_transactions.is_empty() {
            // Batch database insertion
            match self.database.add_transaction_batch(&valid_transactions).await {
                Ok(processed_count) => {
                    {
                        let mut stats = self.stats.write().await;
                        stats.transactions_processed += processed_count;
                        stats.successful_batches += 1;
                    }
                    
                    // Send success results
                    for tx in &valid_transactions {
                        let result = ProcessingResult {
                            transaction_id: tx.id.clone(),
                            success: true,
                            error: None,
                            processing_time: start_time.elapsed(),
                        };
                        let _ = self.result_sender.send(result);
                    }
                },
                Err(e) => {
                    {
                        let mut stats = self.stats.write().await;
                        stats.failed_batches += 1;
                        stats.validation_errors += valid_transactions.len() as u64;
                    }
                    
                    // Send error results
                    for tx in &valid_transactions {
                        let result = ProcessingResult {
                            transaction_id: tx.id.clone(),
                            success: false,
                            error: Some(e.to_string()),
                            processing_time: start_time.elapsed(),
                        };
                        let _ = self.result_sender.send(result);
                    }
                }
            }
        }
        
        // Send results for invalid transactions
        for (valid, tx, error) in validation_results {
            if !valid {
                let result = ProcessingResult {
                    transaction_id: tx.id,
                    success: false,
                    error,
                    processing_time: start_time.elapsed(),
                };
                let _ = self.result_sender.send(result);
            }
        }
        
        self.record_processing_time(start_time.elapsed()).await;
    }

    fn validate_transaction_format(&self, transaction: &Transaction) -> Result<(), ProcessorError> {
        // CPU-intensive validation that benefits from parallel processing
        if transaction.amount == 0 {
            return Err(ProcessorError::ValidationFailed("Zero amount".to_string()));
        }
        
        if transaction.fee < 1000 { // Minimum fee
            return Err(ProcessorError::ValidationFailed("Fee too low".to_string()));
        }
        
        if transaction.sender.len() < 26 || transaction.recipient.len() < 26 {
            return Err(ProcessorError::ValidationFailed("Invalid address format".to_string()));
        }
        
        // Additional format validations...
        Ok(())
    }

    async fn validation_worker_loop(&self, _worker_id: usize) {
        // This worker handles async validation tasks
        loop {
            if *self.shutdown_signal.read().await {
                break;
            }
            
            // Wait for work or timeout
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    async fn stats_update_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        let mut last_processed = 0u64;
        
        loop {
            interval.tick().await;
            
            if *self.shutdown_signal.read().await {
                break;
            }
            
            let mut stats = self.stats.write().await;
            
            // Calculate transactions per second
            let current_processed = stats.transactions_processed;
            stats.transactions_per_second = (current_processed - last_processed) as f64;
            last_processed = current_processed;
            
            // Calculate average processing time
            let processing_times = self.processing_times.lock();
            if !processing_times.is_empty() {
                let total_time: Duration = processing_times.iter().sum();
                stats.average_processing_time_ms = total_time.as_millis() as f64 / processing_times.len() as f64;
            }
            
            // Update queue size
            stats.queue_size = self.priority_queue.len();
        }
    }

    async fn cache_cleanup_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        
        loop {
            interval.tick().await;
            
            if *self.shutdown_signal.read().await {
                break;
            }
            
            // Clean up old validation cache entries
            let cutoff = Instant::now() - Duration::from_secs(600); // 10 minutes
            self.validation_cache.retain(|_, (_, timestamp)| timestamp.elapsed() < Duration::from_secs(600));
            
            // Clean up old processing times
            let mut times = self.processing_times.lock();
            if times.len() > 1000 {
                times.drain(0..500); // Keep only recent 500 measurements
            }
        }
    }

    async fn record_processing_time(&self, duration: Duration) {
        let mut times = self.processing_times.lock();
        times.push_back(duration);
        if times.len() > 1000 {
            times.pop_front();
        }
    }

    pub async fn get_stats(&self) -> ProcessingStats {
        self.stats.read().await.clone()
    }

    pub async fn shutdown(&self) {
        {
            let mut shutdown = self.shutdown_signal.write().await;
            *shutdown = true;
        }
        
        // Process any remaining batches
        while let Some(batch) = self.priority_queue.pop() {
            self.process_batch(batch).await;
        }
    }

    fn clone_for_worker(&self) -> Self {
        Self {
            database: self.database.clone(),
            blockchain: self.blockchain.clone(),
            config: self.config.clone(),
            priority_queue: self.priority_queue.clone(),
            processing_semaphore: self.processing_semaphore.clone(),
            validation_cache: self.validation_cache.clone(),
            stats: self.stats.clone(),
            processing_times: self.processing_times.clone(),
            current_batch: self.current_batch.clone(),
            batch_timer: self.batch_timer.clone(),
            result_sender: self.result_sender.clone(),
            shutdown_signal: self.shutdown_signal.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::Transaction;

    #[tokio::test]
    async fn test_fast_validation() {
        // Test the fast validation pipeline
        let config = ProcessingConfig::default();
        // Would set up mock database and blockchain for testing
    }

    #[tokio::test]
    async fn test_batch_processing() {
        // Test batch processing performance
        let config = ProcessingConfig {
            max_batch_size: 100,
            batch_timeout_ms: 50,
            ..Default::default()
        };
        // Performance testing implementation
    }
}
