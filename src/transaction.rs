use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use anyhow::{Result, anyhow};

pub use crate::blockchain::Transaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInput {
    pub previous_output: String,
    pub script_sig: Vec<u8>,
    pub sequence: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionOutput {
    pub value: u64,
    pub script_pubkey: Vec<u8>,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub id: String,
    pub version: u32,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub lock_time: u32,
    pub timestamp: DateTime<Utc>,
    pub signature: String,
    pub public_key: String,
}

impl SignedTransaction {
    pub fn new(
        inputs: Vec<TransactionInput>,
        outputs: Vec<TransactionOutput>,
        lock_time: u32,
    ) -> Self {
        let id = Self::generate_id(&inputs, &outputs, lock_time);
        
        Self {
            id,
            version: 1,
            inputs,
            outputs,
            lock_time,
            timestamp: Utc::now(),
            signature: String::new(),
            public_key: String::new(),
        }
    }

    pub fn generate_id(
        inputs: &[TransactionInput],
        outputs: &[TransactionOutput],
        lock_time: u32,
    ) -> String {
        let mut data = String::new();
        
        for input in inputs {
            data.push_str(&input.previous_output);
            data.push_str(&hex::encode(&input.script_sig));
        }
        
        for output in outputs {
            data.push_str(&output.value.to_string());
            data.push_str(&hex::encode(&output.script_pubkey));
            data.push_str(&output.address);
        }
        
        data.push_str(&lock_time.to_string());
        
        let hash = blake3::hash(data.as_bytes());
        hex::encode(hash.as_bytes())
    }

    pub fn calculate_fee(&self, utxo_set: &HashMap<String, u64>) -> Result<u64> {
        let mut input_value = 0u64;
        
        for input in &self.inputs {
            match utxo_set.get(&input.previous_output) {
                Some(value) => {
                    input_value = input_value.checked_add(*value)
                        .ok_or_else(|| anyhow!("Input value overflow"))?;
                }
                None => return Err(anyhow!("UTXO not found: {}", input.previous_output)),
            }
        }
        
        let output_value: u64 = self.outputs.iter().map(|o| o.value).sum();
        
        if input_value < output_value {
            return Err(anyhow!("Insufficient funds"));
        }
        
        Ok(input_value - output_value)
    }

    pub fn sign(&mut self, private_key: &str) -> Result<()> {
        let message = self.get_signing_message();
        
        // Use proper post-quantum cryptography for signing
        let quantum_signature = crate::quantum_crypto::sign_message(private_key, message.as_bytes())?;
        self.signature = quantum_signature.signature;
        self.public_key = quantum_signature.public_key;
        
        Ok(())
    }

    pub fn verify_signature(&self, public_key: &str) -> bool {
        if self.signature.is_empty() {
            return false;
        }
        
        let message = self.get_signing_message();
        
        // Use proper post-quantum cryptography verification
        let quantum_signature = crate::quantum_crypto::QuantumSignature {
            signature: self.signature.clone(),
            public_key: public_key.to_string(),
            message_hash: hex::encode(blake3::hash(message.as_bytes()).as_bytes()),
        };
        
        crate::quantum_crypto::verify_signature(&quantum_signature, message.as_bytes())
    }

    fn get_signing_message(&self) -> String {
        let mut message = format!("{}:{}", self.version, self.lock_time);
        
        for input in &self.inputs {
            message.push_str(&format!(
                "{}:{}:{}",
                input.previous_output,
                hex::encode(&input.script_sig),
                input.sequence
            ));
        }
        
        for output in &self.outputs {
            message.push_str(&format!(
                "{}:{}:{}",
                output.value,
                hex::encode(&output.script_pubkey),
                output.address
            ));
        }
        
        let hash = blake3::hash(message.as_bytes());
        hex::encode(hash.as_bytes())
    }

    pub fn to_simple_transaction(&self) -> Transaction {
        let total_output = self.outputs.iter().map(|o| o.value).sum::<u64>();
        let fee = self.calculate_fee(&HashMap::new()).unwrap_or(0);
        
        Transaction {
            id: self.id.clone(),
            from: "".to_string(), // Would need to derive from inputs
            to: self.outputs.first().map(|o| o.address.clone()).unwrap_or_default(),
            amount: total_output,
            timestamp: self.timestamp,
            signature: self.signature.clone(),
            fee,
        }
    }
}

pub struct TransactionPool {
    transactions: HashMap<String, SignedTransaction>,
    max_size: usize,
}

impl TransactionPool {
    pub fn new(max_size: usize) -> Self {
        Self {
            transactions: HashMap::new(),
            max_size,
        }
    }

    pub fn add_transaction(&mut self, transaction: SignedTransaction) -> Result<()> {
        if self.transactions.len() >= self.max_size {
            return Err(anyhow!("Transaction pool is full"));
        }

        if self.transactions.contains_key(&transaction.id) {
            return Err(anyhow!("Transaction already exists in pool"));
        }

        self.transactions.insert(transaction.id.clone(), transaction);
        Ok(())
    }

    pub fn remove_transaction(&mut self, id: &str) -> Option<SignedTransaction> {
        self.transactions.remove(id)
    }

    pub fn get_transaction(&self, id: &str) -> Option<&SignedTransaction> {
        self.transactions.get(id)
    }

    pub fn get_all_transactions(&self) -> Vec<&SignedTransaction> {
        self.transactions.values().collect()
    }

    pub fn size(&self) -> usize {
        self.transactions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }

    pub fn clear(&mut self) {
        self.transactions.clear();
    }
}

impl Default for TransactionPool {
    fn default() -> Self {
        Self::new(10000)
    }
}
