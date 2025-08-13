use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};
use uuid::Uuid;
use base64::{encode, decode};
use pqcrypto_dilithium::dilithium2::{DetachedSignature, PublicKey};
use pqcrypto_traits::sign::Verifier;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Insufficient balance")]
    InsufficientBalance,
    #[error("Invalid amount")]
    InvalidAmount,
    #[error("Invalid address format")]
    InvalidAddress,
    #[error("Transaction fee too low")]
    FeeTooLow,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    pub id: String,
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub fee: u64,
    pub timestamp: DateTime<Utc>,
    pub signature: Option<String>,
    pub public_key: Option<String>,
    pub nonce: u64,
}

impl Transaction {
    pub fn new(sender: String, recipient: String, amount: u64, fee: u64, nonce: u64) -> Self {
        Transaction {
            id: Uuid::new_v4().to_string(),
            sender,
            recipient,
            amount,
            fee,
            timestamp: Utc::now(),
            signature: None,
            public_key: None,
            nonce,
        }
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.sender.as_bytes());
        hasher.update(self.recipient.as_bytes());
        hasher.update(self.amount.to_be_bytes());
        hasher.update(self.fee.to_be_bytes());
        hasher.update(self.timestamp.timestamp().to_be_bytes());
        hasher.update(self.nonce.to_be_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn calculate_fee(&self) -> u64 {
        // Base fee + size-based fee
        let base_fee = 1000; // satoshis
        let size_fee = (self.to_bytes().len() as u64) * 10;
        base_fee + size_fee
    }

    pub fn sign(&mut self, signature: String, public_key: String) {
        self.signature = Some(signature);
        self.public_key = Some(public_key);
    }

    pub fn is_valid(&self, sender_balance: u64) -> Result<bool, TransactionError> {
        // System transactions are always valid
        if self.sender == "GENESIS" || self.sender == "MINING_REWARD" || self.sender == "SYSTEM" {
            return Ok(true);
        }

        // Check amount
        if self.amount == 0 {
            return Err(TransactionError::InvalidAmount);
        }

        // Check balance
        if sender_balance < (self.amount + self.fee) {
            return Err(TransactionError::InsufficientBalance);
        }

        // Check fee
        if self.fee < self.calculate_fee() {
            return Err(TransactionError::FeeTooLow);
        }

        // Verify signature
        self.verify_signature()
    }

    pub fn verify_signature(&self) -> Result<bool, TransactionError> {
        let signature_str = self.signature.as_ref().ok_or(TransactionError::InvalidSignature)?;
        let public_key_str = self.public_key.as_ref().ok_or(TransactionError::InvalidSignature)?;

        let signature_bytes = decode(signature_str).map_err(|_| TransactionError::InvalidSignature)?;
        let public_key_bytes = decode(public_key_str).map_err(|_| TransactionError::InvalidSignature)?;

        let signature = DetachedSignature::from_bytes(&signature_bytes)
            .map_err(|_| TransactionError::InvalidSignature)?;
        let public_key = PublicKey::from_bytes(&public_key_bytes)
            .map_err(|_| TransactionError::InvalidSignature)?;

        let message = self.to_message();
        match signature.verify_detached(message.as_bytes(), &public_key) {
            Ok(_) => Ok(true),
            Err(_) => Err(TransactionError::InvalidSignature),
        }
    }

    pub fn to_message(&self) -> String {
        format!("{}:{}:{}:{}:{}:{}", 
            self.sender, self.recipient, self.amount, 
            self.fee, self.timestamp.timestamp(), self.nonce)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_default()
    }

    pub fn total_cost(&self) -> u64 {
        self.amount + self.fee
    }
}