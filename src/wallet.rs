use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use crate::quantum_crypto::{QuantumTransactionSigner, generate_keypair, public_key_to_address};
use crate::transaction::{SignedTransaction, TransactionInput, TransactionOutput};
use crate::blockchain::Blockchain;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub address: String,
    pub public_key: String,
    private_key: String,
    pub balance: u64,
    pub transaction_history: Vec<WalletTransaction>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletTransaction {
    pub transaction_id: String,
    pub transaction_type: TransactionType,
    pub amount: u64,
    pub fee: u64,
    pub counterpart_address: String,
    pub timestamp: DateTime<Utc>,
    pub confirmations: u32,
    pub status: TransactionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Sent,
    Received,
    SelfTransfer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Reversed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXO {
    pub transaction_id: String,
    pub output_index: u32,
    pub amount: u64,
    pub address: String,
    pub confirmations: u32,
    pub is_coinbase: bool,
}

impl Wallet {
    pub fn new() -> Result<Self> {
        let (public_key, private_key) = generate_keypair();
        let address = public_key_to_address(&public_key);
        
        Ok(Self {
            address,
            public_key,
            private_key,
            balance: 0,
            transaction_history: Vec::new(),
            created_at: Utc::now(),
        })
    }

    pub fn from_private_key(private_key: String) -> Result<Self> {
        let signer = QuantumTransactionSigner::from_private_key(private_key.clone())?;
        let public_key = signer.get_public_key().to_string();
        let address = signer.get_address();
        
        Ok(Self {
            address,
            public_key,
            private_key,
            balance: 0,
            transaction_history: Vec::new(),
            created_at: Utc::now(),
        })
    }

    pub fn get_address(&self) -> &str {
        &self.address
    }

    pub fn get_public_key(&self) -> &str {
        &self.public_key
    }

    pub fn get_balance(&self) -> u64 {
        self.balance
    }

    pub fn update_balance(&mut self, blockchain: &Blockchain) {
        self.balance = blockchain.get_balance(&self.address);
    }

    pub fn create_transaction(
        &self,
        to_address: String,
        amount: u64,
        fee: u64,
        utxos: Vec<UTXO>,
    ) -> Result<SignedTransaction> {
        if amount == 0 {
            return Err(anyhow!("Amount must be greater than zero"));
        }

        if fee == 0 {
            return Err(anyhow!("Fee must be greater than zero"));
        }

        // Select UTXOs to cover the transaction
        let selected_utxos = self.select_utxos(utxos, amount + fee)?;
        let total_input: u64 = selected_utxos.iter().map(|utxo| utxo.amount).sum();

        if total_input < amount + fee {
            return Err(anyhow!("Insufficient funds"));
        }

        // Create inputs from selected UTXOs
        let inputs: Vec<TransactionInput> = selected_utxos
            .iter()
            .map(|utxo| TransactionInput {
                previous_output: format!("{}:{}", utxo.transaction_id, utxo.output_index),
                script_sig: vec![], // Will be filled with signature
                sequence: 0xFFFFFFFF,
            })
            .collect();

        // Create outputs
        let mut outputs = vec![TransactionOutput {
            value: amount,
            script_pubkey: to_address.as_bytes().to_vec(),
            address: to_address,
        }];

        // Add change output if needed
        let change_amount = total_input - amount - fee;
        if change_amount > 0 {
            outputs.push(TransactionOutput {
                value: change_amount,
                script_pubkey: self.address.as_bytes().to_vec(),
                address: self.address.clone(),
            });
        }

        // Create and sign transaction
        let mut transaction = SignedTransaction::new(inputs, outputs, 0);
        
        let signer = QuantumTransactionSigner::from_private_key(self.private_key.clone())?;
        let transaction_data = self.serialize_for_signing(&transaction)?;
        let signature = signer.sign_transaction(&transaction_data)?;
        
        transaction.signature = signature.signature;
        transaction.public_key = self.public_key.clone();

        Ok(transaction)
    }

    fn select_utxos(&self, available_utxos: Vec<UTXO>, required_amount: u64) -> Result<Vec<UTXO>> {
        let mut sorted_utxos = available_utxos;
        // Sort by amount descending (largest first strategy)
        sorted_utxos.sort_by(|a, b| b.amount.cmp(&a.amount));

        let mut selected = Vec::new();
        let mut total = 0u64;

        for utxo in sorted_utxos {
            // Skip unconfirmed coinbase transactions (need 100 confirmations)
            if utxo.is_coinbase && utxo.confirmations < 100 {
                continue;
            }

            selected.push(utxo);
            total += utxo.amount;

            if total >= required_amount {
                break;
            }
        }

        if total < required_amount {
            return Err(anyhow!("Insufficient confirmed funds"));
        }

        Ok(selected)
    }

    fn serialize_for_signing(&self, transaction: &SignedTransaction) -> Result<Vec<u8>> {
        // Create a simplified transaction for signing (without signatures)
        let mut signing_data = Vec::new();
        
        // Add version
        signing_data.extend_from_slice(&transaction.version.to_le_bytes());
        
        // Add inputs (without script_sig)
        for input in &transaction.inputs {
            signing_data.extend_from_slice(input.previous_output.as_bytes());
            signing_data.extend_from_slice(&input.sequence.to_le_bytes());
        }
        
        // Add outputs
        for output in &transaction.outputs {
            signing_data.extend_from_slice(&output.value.to_le_bytes());
            signing_data.extend_from_slice(&output.address.as_bytes());
        }
        
        // Add lock time
        signing_data.extend_from_slice(&transaction.lock_time.to_le_bytes());
        
        Ok(signing_data)
    }

    pub fn add_transaction(&mut self, transaction: WalletTransaction) {
        self.transaction_history.push(transaction);
        
        // Keep only last 1000 transactions
        if self.transaction_history.len() > 1000 {
            self.transaction_history.remove(0);
        }
    }

    pub fn get_transaction_history(&self) -> &[WalletTransaction] {
        &self.transaction_history
    }

    pub fn get_pending_transactions(&self) -> Vec<&WalletTransaction> {
        self.transaction_history
            .iter()
            .filter(|tx| tx.status == TransactionStatus::Pending)
            .collect()
    }

    pub fn update_transaction_status(&mut self, transaction_id: &str, status: TransactionStatus, confirmations: u32) {
        if let Some(tx) = self.transaction_history
            .iter_mut()
            .find(|tx| tx.transaction_id == transaction_id) {
            tx.status = status;
            tx.confirmations = confirmations;
        }
    }

    pub fn export_private_key(&self) -> &str {
        &self.private_key
    }

    pub fn export_wallet(&self) -> Result<String> {
        let export_data = WalletExport {
            address: self.address.clone(),
            public_key: self.public_key.clone(),
            private_key: self.private_key.clone(),
            created_at: self.created_at,
        };
        
        Ok(serde_json::to_string_pretty(&export_data)?)
    }

    pub fn import_wallet(wallet_data: &str) -> Result<Self> {
        let export_data: WalletExport = serde_json::from_str(wallet_data)?;
        
        let mut wallet = Self::from_private_key(export_data.private_key)?;
        wallet.created_at = export_data.created_at;
        
        // Verify the imported data matches
        if wallet.address != export_data.address || wallet.public_key != export_data.public_key {
            return Err(anyhow!("Wallet import verification failed"));
        }
        
        Ok(wallet)
    }

    pub fn estimate_fee(&self, utxos: &[UTXO], outputs_count: usize) -> u64 {
        // Simple fee calculation: base fee + input fees + output fees
        let base_fee = 1000; // 0.00001 QTC
        let input_fee = 5000 * utxos.len() as u64; // 0.00005 QTC per input
        let output_fee = 2000 * outputs_count as u64; // 0.00002 QTC per output
        
        base_fee + input_fee + output_fee
    }

    pub fn get_wallet_stats(&self) -> WalletStats {
        let total_sent = self.transaction_history
            .iter()
            .filter(|tx| matches!(tx.transaction_type, TransactionType::Sent))
            .map(|tx| tx.amount + tx.fee)
            .sum();

        let total_received = self.transaction_history
            .iter()
            .filter(|tx| matches!(tx.transaction_type, TransactionType::Received))
            .map(|tx| tx.amount)
            .sum();

        let total_fees = self.transaction_history
            .iter()
            .map(|tx| tx.fee)
            .sum();

        WalletStats {
            balance: self.balance,
            total_sent,
            total_received,
            total_fees,
            transaction_count: self.transaction_history.len(),
            pending_transactions: self.get_pending_transactions().len(),
            created_at: self.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WalletExport {
    address: String,
    public_key: String,
    private_key: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletStats {
    pub balance: u64,
    pub total_sent: u64,
    pub total_received: u64,
    pub total_fees: u64,
    pub transaction_count: usize,
    pub pending_transactions: usize,
    pub created_at: DateTime<Utc>,
}

pub struct WalletManager {
    wallets: HashMap<String, Wallet>,
    active_wallet: Option<String>,
}

impl WalletManager {
    pub fn new() -> Self {
        Self {
            wallets: HashMap::new(),
            active_wallet: None,
        }
    }

    pub fn create_wallet(&mut self) -> Result<String> {
        let wallet = Wallet::new()?;
        let address = wallet.address.clone();
        
        self.wallets.insert(address.clone(), wallet);
        
        if self.active_wallet.is_none() {
            self.active_wallet = Some(address.clone());
        }
        
        Ok(address)
    }

    pub fn import_wallet(&mut self, private_key: String) -> Result<String> {
        let wallet = Wallet::from_private_key(private_key)?;
        let address = wallet.address.clone();
        
        self.wallets.insert(address.clone(), wallet);
        
        Ok(address)
    }

    pub fn get_wallet(&self, address: &str) -> Option<&Wallet> {
        self.wallets.get(address)
    }

    pub fn get_wallet_mut(&mut self, address: &str) -> Option<&mut Wallet> {
        self.wallets.get_mut(address)
    }

    pub fn get_active_wallet(&self) -> Option<&Wallet> {
        self.active_wallet.as_ref().and_then(|addr| self.wallets.get(addr))
    }

    pub fn get_active_wallet_mut(&mut self) -> Option<&mut Wallet> {
        let active_addr = self.active_wallet.clone()?;
        self.wallets.get_mut(&active_addr)
    }

    pub fn set_active_wallet(&mut self, address: String) -> Result<()> {
        if !self.wallets.contains_key(&address) {
            return Err(anyhow!("Wallet not found"));
        }
        
        self.active_wallet = Some(address);
        Ok(())
    }

    pub fn list_wallets(&self) -> Vec<&Wallet> {
        self.wallets.values().collect()
    }

    pub fn remove_wallet(&mut self, address: &str) -> Result<()> {
        if !self.wallets.contains_key(address) {
            return Err(anyhow!("Wallet not found"));
        }
        
        self.wallets.remove(address);
        
        if self.active_wallet.as_ref() == Some(&address.to_string()) {
            self.active_wallet = self.wallets.keys().next().cloned();
        }
        
        Ok(())
    }

    pub fn update_all_balances(&mut self, blockchain: &Blockchain) {
        for wallet in self.wallets.values_mut() {
            wallet.update_balance(blockchain);
        }
    }
}

impl Default for WalletManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = Wallet::new().unwrap();
        assert!(!wallet.address.is_empty());
        assert!(!wallet.public_key.is_empty());
        assert_eq!(wallet.balance, 0);
    }

    #[test]
    fn test_wallet_export_import() {
        let original_wallet = Wallet::new().unwrap();
        let exported = original_wallet.export_wallet().unwrap();
        let imported_wallet = Wallet::import_wallet(&exported).unwrap();
        
        assert_eq!(original_wallet.address, imported_wallet.address);
        assert_eq!(original_wallet.public_key, imported_wallet.public_key);
    }

    #[test]
    fn test_wallet_manager() {
        let mut manager = WalletManager::new();
        let address1 = manager.create_wallet().unwrap();
        let address2 = manager.create_wallet().unwrap();
        
        assert_eq!(manager.list_wallets().len(), 2);
        assert!(manager.get_wallet(&address1).is_some());
        assert!(manager.get_wallet(&address2).is_some());
    }

    #[test]
    fn test_utxo_selection() {
        let wallet = Wallet::new().unwrap();
        let utxos = vec![
            UTXO {
                transaction_id: "tx1".to_string(),
                output_index: 0,
                amount: 1000,
                address: wallet.address.clone(),
                confirmations: 6,
                is_coinbase: false,
            },
            UTXO {
                transaction_id: "tx2".to_string(),
                output_index: 0,
                amount: 2000,
                address: wallet.address.clone(),
                confirmations: 6,
                is_coinbase: false,
            },
        ];
        
        let selected = wallet.select_utxos(utxos, 1500).unwrap();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].amount, 2000); // Should select the larger UTXO
    }
}
