use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};
use tracing::{info, warn, error};

use crate::blockchain::Block;
use crate::transaction::{Transaction, SignedTransaction};

/// Revolutionary Stop-Loss mechanism for QuantumCoin
/// Provides automatic transaction reversals, fraud detection, and value protection
#[derive(Debug, Clone)]
pub struct RevStop {
    /// Active reversals being monitored
    active_reversals: HashMap<String, ReversalOrder>,
    /// Transaction history for analysis
    transaction_history: VecDeque<TransactionAnalysis>,
    /// Fraud detection patterns
    fraud_patterns: Vec<FraudPattern>,
    /// Configuration parameters
    config: RevStopConfig,
    /// Statistics
    stats: RevStopStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReversalOrder {
    pub id: String,
    pub original_transaction_id: String,
    pub reversal_transaction_id: Option<String>,
    pub from_address: String,
    pub to_address: String,
    pub amount: u64,
    pub reason: ReversalReason,
    pub status: ReversalStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub confidence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReversalReason {
    FraudDetected,
    UnauthorizedAccess,
    SystemError,
    UserRequested,
    ComplianceViolation,
    QuantumThreat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReversalStatus {
    Pending,
    Approved,
    Executed,
    Rejected,
    Expired,
}

#[derive(Debug, Clone)]
pub struct TransactionAnalysis {
    pub transaction_id: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: u64,
    pub timestamp: DateTime<Utc>,
    pub risk_score: f64,
    pub behavioral_score: f64,
    pub quantum_threat_level: u8,
}

#[derive(Debug, Clone)]
pub struct FraudPattern {
    pub pattern_id: String,
    pub description: String,
    pub detection_rules: Vec<DetectionRule>,
    pub severity: u8, // 1-10
    pub confidence_threshold: f64,
}

#[derive(Debug, Clone)]
pub enum DetectionRule {
    VelocityAnomaly { threshold: u64, window_minutes: u32 },
    AmountAnomaly { deviation_factor: f64 },
    GeographicAnomaly { suspicious_regions: Vec<String> },
    BehavioralAnomaly { pattern_deviation: f64 },
    QuantumSignatureAnomaly { entropy_threshold: f64 },
}

#[derive(Debug, Clone)]
pub struct RevStopConfig {
    pub reversal_window_hours: u32,
    pub min_confidence_score: f64,
    pub max_reversal_amount: u64,
    pub fraud_detection_enabled: bool,
    pub auto_reversal_threshold: f64,
    pub quantum_threat_monitoring: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevStopStats {
    pub total_reversals: u64,
    pub successful_reversals: u64,
    pub fraud_cases_detected: u64,
    pub false_positives: u64,
    pub average_detection_time: u64, // seconds
    pub quantum_threats_blocked: u64,
}

impl RevStop {
    pub fn new() -> Self {
        Self {
            active_reversals: HashMap::new(),
            transaction_history: VecDeque::new(),
            fraud_patterns: Self::initialize_fraud_patterns(),
            config: RevStopConfig::default(),
            stats: RevStopStats::default(),
        }
    }

    /// Initialize fraud detection patterns
    fn initialize_fraud_patterns() -> Vec<FraudPattern> {
        vec![
            FraudPattern {
                pattern_id: "velocity_attack".to_string(),
                description: "Rapid succession of high-value transactions".to_string(),
                detection_rules: vec![
                    DetectionRule::VelocityAnomaly { 
                        threshold: 1_000_000_000, // 10 QTC
                        window_minutes: 5 
                    }
                ],
                severity: 8,
                confidence_threshold: 0.85,
            },
            FraudPattern {
                pattern_id: "quantum_signature_forgery".to_string(),
                description: "Potential quantum computer attack on signatures".to_string(),
                detection_rules: vec![
                    DetectionRule::QuantumSignatureAnomaly { 
                        entropy_threshold: 0.7 
                    }
                ],
                severity: 10,
                confidence_threshold: 0.9,
            },
            FraudPattern {
                pattern_id: "behavioral_anomaly".to_string(),
                description: "Transaction pattern doesn't match user behavior".to_string(),
                detection_rules: vec![
                    DetectionRule::BehavioralAnomaly { 
                        pattern_deviation: 3.0 
                    }
                ],
                severity: 6,
                confidence_threshold: 0.75,
            }
        ]
    }

    /// Analyze a transaction for potential fraud
    pub async fn analyze_transaction(&mut self, transaction: &SignedTransaction) -> Result<TransactionAnalysis> {
        let analysis = TransactionAnalysis {
            transaction_id: transaction.id.clone(),
            from_address: "".to_string(), // Would extract from inputs
            to_address: transaction.outputs.first().map(|o| o.address.clone()).unwrap_or_default(),
            amount: transaction.outputs.iter().map(|o| o.value).sum(),
            timestamp: transaction.timestamp,
            risk_score: self.calculate_risk_score(transaction).await,
            behavioral_score: self.calculate_behavioral_score(transaction).await,
            quantum_threat_level: self.assess_quantum_threat(transaction).await,
        };

        // Add to history (keep last 10000 transactions)
        self.transaction_history.push_back(analysis.clone());
        if self.transaction_history.len() > 10000 {
            self.transaction_history.pop_front();
        }

        // Check if automatic reversal is needed
        if analysis.risk_score > self.config.auto_reversal_threshold {
            self.create_automatic_reversal(&analysis).await?;
        }

        Ok(analysis)
    }

    async fn calculate_risk_score(&self, transaction: &SignedTransaction) -> f64 {
        let mut risk_score = 0.0;

        // Check against fraud patterns
        for pattern in &self.fraud_patterns {
            let pattern_risk = self.evaluate_fraud_pattern(transaction, pattern).await;
            risk_score = risk_score.max(pattern_risk);
        }

        // Amount-based risk
        let amount = transaction.outputs.iter().map(|o| o.value).sum::<u64>();
        if amount > 10_000_000_000 { // > 100 QTC
            risk_score += 0.3;
        }

        // Velocity check
        let velocity_risk = self.check_velocity_risk(transaction).await;
        risk_score += velocity_risk;

        risk_score.min(1.0)
    }

    async fn calculate_behavioral_score(&self, _transaction: &SignedTransaction) -> f64 {
        // Simplified behavioral scoring
        // In a real system, this would analyze user patterns, timing, amounts, etc.
        0.5
    }

    async fn assess_quantum_threat(&self, transaction: &SignedTransaction) -> u8 {
        if !self.config.quantum_threat_monitoring {
            return 0;
        }

        // Analyze quantum signature entropy
        let signature_entropy = self.calculate_signature_entropy(&transaction.signature);
        
        if signature_entropy < 0.7 {
            10 // High quantum threat
        } else if signature_entropy < 0.8 {
            5  // Medium quantum threat
        } else {
            0  // Low quantum threat
        }
    }

    fn calculate_signature_entropy(&self, signature: &str) -> f64 {
        if signature.is_empty() {
            return 0.0;
        }

        let bytes = signature.as_bytes();
        let mut frequency = [0u32; 256];
        
        for &byte in bytes {
            frequency[byte as usize] += 1;
        }

        let len = bytes.len() as f64;
        let mut entropy = 0.0;
        
        for &freq in frequency.iter() {
            if freq > 0 {
                let p = freq as f64 / len;
                entropy -= p * p.log2();
            }
        }

        entropy / 8.0 // Normalize to 0-1 range
    }

    async fn evaluate_fraud_pattern(&self, transaction: &SignedTransaction, pattern: &FraudPattern) -> f64 {
        let mut pattern_score = 0.0;

        for rule in &pattern.detection_rules {
            match rule {
                DetectionRule::VelocityAnomaly { threshold, window_minutes } => {
                    let velocity_score = self.check_velocity_anomaly(transaction, *threshold, *window_minutes).await;
                    pattern_score = pattern_score.max(velocity_score);
                }
                DetectionRule::QuantumSignatureAnomaly { entropy_threshold } => {
                    let entropy = self.calculate_signature_entropy(&transaction.signature);
                    if entropy < *entropy_threshold {
                        pattern_score = pattern_score.max(0.9);
                    }
                }
                DetectionRule::BehavioralAnomaly { pattern_deviation } => {
                    // Simplified behavioral check
                    let deviation = self.calculate_behavioral_deviation(transaction).await;
                    if deviation > *pattern_deviation {
                        pattern_score = pattern_score.max(0.7);
                    }
                }
                _ => {} // Other rules not implemented yet
            }
        }

        pattern_score * (pattern.severity as f64 / 10.0)
    }

    async fn check_velocity_anomaly(&self, transaction: &SignedTransaction, threshold: u64, window_minutes: u32) -> f64 {
        let window_start = transaction.timestamp - Duration::minutes(window_minutes as i64);
        let total_amount: u64 = self.transaction_history
            .iter()
            .filter(|tx| tx.timestamp > window_start)
            .map(|tx| tx.amount)
            .sum();

        if total_amount > threshold {
            0.8 // High velocity risk
        } else {
            0.0
        }
    }

    async fn check_velocity_risk(&self, transaction: &SignedTransaction) -> f64 {
        // Check transactions in last 10 minutes
        let window_start = transaction.timestamp - Duration::minutes(10);
        let recent_count = self.transaction_history
            .iter()
            .filter(|tx| tx.timestamp > window_start)
            .count();

        if recent_count > 10 {
            0.6 // High velocity
        } else if recent_count > 5 {
            0.3 // Medium velocity
        } else {
            0.0 // Normal velocity
        }
    }

    async fn calculate_behavioral_deviation(&self, _transaction: &SignedTransaction) -> f64 {
        // Simplified implementation
        // Would analyze user's historical patterns
        1.5
    }

    async fn create_automatic_reversal(&mut self, analysis: &TransactionAnalysis) -> Result<()> {
        let reversal_id = format!("rev_{}", uuid::Uuid::new_v4());
        
        let reversal_order = ReversalOrder {
            id: reversal_id,
            original_transaction_id: analysis.transaction_id.clone(),
            reversal_transaction_id: None,
            from_address: analysis.from_address.clone(),
            to_address: analysis.to_address.clone(),
            amount: analysis.amount,
            reason: if analysis.quantum_threat_level > 7 {
                ReversalReason::QuantumThreat
            } else {
                ReversalReason::FraudDetected
            },
            status: ReversalStatus::Pending,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(self.config.reversal_window_hours as i64),
            confidence_score: analysis.risk_score,
        };

        info!(
            "Creating automatic reversal for transaction {} with confidence {}",
            analysis.transaction_id, analysis.risk_score
        );

        self.active_reversals.insert(reversal_order.id.clone(), reversal_order);
        self.stats.total_reversals += 1;

        Ok(())
    }

    /// Manual reversal request
    pub async fn request_reversal(
        &mut self,
        transaction_id: String,
        reason: ReversalReason,
        requester: String,
    ) -> Result<String> {
        // Check if reversal window is still open
        let transaction = self.find_transaction_analysis(&transaction_id)
            .ok_or_else(|| anyhow!("Transaction not found"))?;

        let now = Utc::now();
        let window_end = transaction.timestamp + Duration::hours(self.config.reversal_window_hours as i64);
        
        if now > window_end {
            return Err(anyhow!("Reversal window expired"));
        }

        let reversal_id = format!("rev_{}", uuid::Uuid::new_v4());
        
        let reversal_order = ReversalOrder {
            id: reversal_id.clone(),
            original_transaction_id: transaction_id,
            reversal_transaction_id: None,
            from_address: transaction.from_address.clone(),
            to_address: transaction.to_address.clone(),
            amount: transaction.amount,
            reason,
            status: ReversalStatus::Pending,
            created_at: now,
            expires_at: window_end,
            confidence_score: 1.0, // Manual requests have high confidence
        };

        info!("Manual reversal requested by {}: {}", requester, reversal_id);

        self.active_reversals.insert(reversal_id.clone(), reversal_order);
        self.stats.total_reversals += 1;

        Ok(reversal_id)
    }

    /// Execute approved reversals
    pub async fn process_reversals(&mut self) -> Result<Vec<String>> {
        let mut executed_reversals = Vec::new();
        let now = Utc::now();

        let expired_reversals: Vec<String> = self.active_reversals
            .iter()
            .filter(|(_, order)| now > order.expires_at && order.status == ReversalStatus::Pending)
            .map(|(id, _)| id.clone())
            .collect();

        // Mark expired reversals
        for reversal_id in expired_reversals {
            if let Some(order) = self.active_reversals.get_mut(&reversal_id) {
                order.status = ReversalStatus::Expired;
                warn!("Reversal expired: {}", reversal_id);
            }
        }

        // Process approved reversals
        let approved_reversals: Vec<String> = self.active_reversals
            .iter()
            .filter(|(_, order)| order.status == ReversalStatus::Approved)
            .map(|(id, _)| id.clone())
            .collect();

        for reversal_id in approved_reversals {
            match self.execute_reversal(&reversal_id).await {
                Ok(tx_id) => {
                    executed_reversals.push(tx_id);
                    self.stats.successful_reversals += 1;
                }
                Err(e) => {
                    error!("Failed to execute reversal {}: {}", reversal_id, e);
                }
            }
        }

        Ok(executed_reversals)
    }

    async fn execute_reversal(&mut self, reversal_id: &str) -> Result<String> {
        let reversal_order = self.active_reversals
            .get_mut(reversal_id)
            .ok_or_else(|| anyhow!("Reversal not found"))?;

        // Create reversal transaction
        let reversal_tx_id = format!("revtx_{}", uuid::Uuid::new_v4());
        
        // In a real implementation, this would create and broadcast a reversal transaction
        info!(
            "Executing reversal: {} QTC from {} to {}",
            reversal_order.amount as f64 / 100_000_000.0,
            reversal_order.to_address,
            reversal_order.from_address
        );

        reversal_order.reversal_transaction_id = Some(reversal_tx_id.clone());
        reversal_order.status = ReversalStatus::Executed;

        Ok(reversal_tx_id)
    }

    pub async fn update_on_new_block(&mut self, block: &Block) -> Result<()> {
        info!("RevStop: Processing new block {}", block.hash);

        // Analyze all transactions in the block
        for transaction in &block.transactions {
            // Convert simple transaction to signed transaction (simplified)
            let signed_tx = self.simple_to_signed_transaction(transaction);
            self.analyze_transaction(&signed_tx).await?;
        }

        // Clean up old reversals
        self.cleanup_old_reversals().await;

        Ok(())
    }

    fn simple_to_signed_transaction(&self, tx: &Transaction) -> SignedTransaction {
        use crate::transaction::{TransactionInput, TransactionOutput};

        SignedTransaction {
            id: tx.id.clone(),
            version: 1,
            inputs: vec![TransactionInput {
                previous_output: "unknown".to_string(),
                script_sig: vec![],
                sequence: 0,
            }],
            outputs: vec![TransactionOutput {
                value: tx.amount,
                script_pubkey: vec![],
                address: tx.to.clone(),
            }],
            lock_time: 0,
            timestamp: tx.timestamp,
            signature: tx.signature.clone(),
            public_key: "unknown".to_string(),
        }
    }

    async fn cleanup_old_reversals(&mut self) {
        let now = Utc::now();
        let cutoff = now - Duration::days(30); // Keep reversals for 30 days

        self.active_reversals.retain(|_, order| order.created_at > cutoff);
        
        // Also clean up transaction history
        while let Some(front) = self.transaction_history.front() {
            if front.timestamp < cutoff {
                self.transaction_history.pop_front();
            } else {
                break;
            }
        }
    }

    fn find_transaction_analysis(&self, transaction_id: &str) -> Option<&TransactionAnalysis> {
        self.transaction_history
            .iter()
            .find(|tx| tx.transaction_id == transaction_id)
    }

    pub fn get_reversal_status(&self, reversal_id: &str) -> Option<&ReversalOrder> {
        self.active_reversals.get(reversal_id)
    }

    pub fn get_active_reversals(&self) -> Vec<&ReversalOrder> {
        self.active_reversals.values().collect()
    }

    pub fn get_stats(&self) -> &RevStopStats {
        &self.stats
    }

    pub fn approve_reversal(&mut self, reversal_id: &str) -> Result<()> {
        let reversal = self.active_reversals
            .get_mut(reversal_id)
            .ok_or_else(|| anyhow!("Reversal not found"))?;

        if reversal.status != ReversalStatus::Pending {
            return Err(anyhow!("Reversal is not in pending status"));
        }

        reversal.status = ReversalStatus::Approved;
        info!("Reversal approved: {}", reversal_id);
        Ok(())
    }

    pub fn reject_reversal(&mut self, reversal_id: &str) -> Result<()> {
        let reversal = self.active_reversals
            .get_mut(reversal_id)
            .ok_or_else(|| anyhow!("Reversal not found"))?;

        if reversal.status != ReversalStatus::Pending {
            return Err(anyhow!("Reversal is not in pending status"));
        }

        reversal.status = ReversalStatus::Rejected;
        self.stats.false_positives += 1;
        info!("Reversal rejected: {}", reversal_id);
        Ok(())
    }
}

impl Default for RevStopConfig {
    fn default() -> Self {
        Self {
            reversal_window_hours: 24,
            min_confidence_score: 0.75,
            max_reversal_amount: 100_000_000_000, // 1000 QTC
            fraud_detection_enabled: true,
            auto_reversal_threshold: 0.9,
            quantum_threat_monitoring: true,
        }
    }
}

impl Default for RevStopStats {
    fn default() -> Self {
        Self {
            total_reversals: 0,
            successful_reversals: 0,
            fraud_cases_detected: 0,
            false_positives: 0,
            average_detection_time: 0,
            quantum_threats_blocked: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{TransactionInput, TransactionOutput};

    fn create_test_transaction() -> SignedTransaction {
        SignedTransaction {
            id: "test_tx".to_string(),
            version: 1,
            inputs: vec![TransactionInput {
                previous_output: "input".to_string(),
                script_sig: vec![],
                sequence: 0,
            }],
            outputs: vec![TransactionOutput {
                value: 1000,
                script_pubkey: vec![],
                address: "test_address".to_string(),
            }],
            lock_time: 0,
            timestamp: Utc::now(),
            signature: "test_signature".to_string(),
            public_key: "test_public_key".to_string(),
        }
    }

    #[tokio::test]
    async fn test_revstop_creation() {
        let revstop = RevStop::new();
        assert_eq!(revstop.active_reversals.len(), 0);
        assert!(revstop.fraud_patterns.len() > 0);
    }

    #[tokio::test]
    async fn test_transaction_analysis() {
        let mut revstop = RevStop::new();
        let tx = create_test_transaction();
        
        let analysis = revstop.analyze_transaction(&tx).await.unwrap();
        assert_eq!(analysis.transaction_id, "test_tx");
        assert!(analysis.risk_score >= 0.0 && analysis.risk_score <= 1.0);
    }

    #[tokio::test]
    async fn test_manual_reversal() {
        let mut revstop = RevStop::new();
        let tx = create_test_transaction();
        
        // First analyze the transaction
        revstop.analyze_transaction(&tx).await.unwrap();
        
        // Then request reversal
        let reversal_id = revstop.request_reversal(
            tx.id.clone(),
            ReversalReason::UserRequested,
            "test_user".to_string(),
        ).await.unwrap();
        
        assert!(!reversal_id.is_empty());
        assert!(revstop.get_reversal_status(&reversal_id).is_some());
    }
}
