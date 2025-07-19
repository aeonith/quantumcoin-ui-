use pqcrypto_dilithium::dilithium2::{keypair, sign_detached, verify_detached, PublicKey, SecretKey, SignedMessage};
use pqcrypto_traits::sign::{PublicKey as TraitPublicKey, SecretKey as TraitSecretKey, SignedMessage as TraitSignedMessage};
use base64::{encode, decode};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use crate::transaction::Transaction;

#[derive(Debug)]
pub struct Wallet {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
    pub balance: f64,
    pub last_transactions: Vec<String>, // stores last 5 TX hashes
}

impl Wallet {
    /// Create a new wallet with fresh Dilithium2 keys
    pub fn new() -> Self {
        let (pk, sk) = keypair();
        Wallet {
            public_key: pk,
            secret_key: sk,
            balance: 0.0,
            last_transactions: vec![],
        }
    }

    /// Save keys and balance to wallet_key.json
    pub fn save_to_files(&self) {
        let pub_key_str = encode(self.public_key.as_bytes());
        let sec_key_str = encode(self.secret_key.as_bytes());
        let tx_log = self.last_transactions.clone();

        let data = serde_json::json!({
            "public_key": pub_key_str,
            "secret_key": sec_key_str,
            "balance": self.balance,
            "last_transactions": tx_log
        });

        let mut file = File::create("wallet_key.json").expect("Unable to create wallet file");
        file.write_all(data.to_string().as_bytes()).expect("Unable to write wallet file");
    }

    /// Load wallet from wallet_key.json
    pub fn load_from_files() -> Self {
        let mut file = File::open("wallet_key.json").expect("wallet_key.json not found");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Failed to read wallet");

        let json: serde_json::Value = serde_json::from_str(&contents).unwrap();
        let pub_key = decode(json["public_key"].as_str().unwrap()).unwrap();
        let sec_key = decode(json["secret_key"].as_str().unwrap()).unwrap();
        let balance = json["balance"].as_f64().unwrap_or(0.0);
        let tx_log = json["last_transactions"].as_array().unwrap_or(&vec![])
            .iter().map(|v| v.as_str().unwrap_or("").to_string()).collect();

        Wallet {
            public_key: PublicKey::from_bytes(&pub_key).unwrap(),
            secret_key: SecretKey::from_bytes(&sec_key).unwrap(),
            balance,
            last_transactions: tx_log,
        }
    }

    /// Return public address as base64 string
    pub fn get_address(&self) -> String {
        encode(self.public_key.as_bytes())
    }

    /// Sign a message (like a transaction)
    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        let signed = sign_detached(message, &self.secret_key);
        signed.as_bytes().to_vec()
    }

    /// Verify a message's signature
    pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> bool {
        let sig = SignedMessage::from_bytes(signature);
        match sig {
            Ok(sig) => verify_detached(&sig, message, &self.public_key).is_ok(),
            Err(_) => false,
        }
    }

    /// Export wallet with 2FA passphrase
    pub fn export_with_2fa(&self, passphrase: &str) {
        let export = serde_json::json!({
            "public_key": encode(self.public_key.as_bytes()),
            "secret_key": encode(self.secret_key.as_bytes()),
            "2fa_passphrase": passphrase
        });

        let mut file = File::create("wallet_export.json").expect("Failed to export wallet");
        file.write_all(export.to_string().as_bytes()).expect("Write failed");
        println!("🔐 Wallet exported with 2FA.");
    }

    /// Create a transaction
    pub fn create_transaction(&mut self, recipient: String, amount: f64) -> Transaction {
        if self.balance < amount {
            panic!("❌ Insufficient balance!");
        }

        self.balance -= amount;

        let tx = Transaction::new(self.get_address(), recipient.clone(), amount, self);
        self.log_transaction(&tx.tx_hash);
        tx
    }

    /// Log last 5 transactions
    pub fn log_transaction(&mut self, tx_hash: &str) {
        self.last_transactions.push(tx_hash.to_string());
        if self.last_transactions.len() > 5 {
            self.last_transactions.remove(0);
        }
        self.save_to_files(); // persist updates
    }

    /// Show recent transactions
    pub fn show_last_transactions(&self) {
        println!("📜 Last 5 Transactions:");
        for tx in self.last_transactions.iter().rev() {
            println!("🔸 {}", tx);
        }
    }

    /// Return current balance
    pub fn get_balance(&self) -> f64 {
        self.balance
    }

    /// Increase balance after mining or receiving
    pub fn add_balance(&mut self, amount: f64) {
        self.balance += amount;
        self.save_to_files();
    }
}