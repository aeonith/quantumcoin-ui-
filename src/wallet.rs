use pqcrypto_dilithium::dilithium2::{keypair, PublicKey, SecretKey, sign_detached, verify_detached};
use pqcrypto_traits::sign::{PublicKey as TraitPublicKey, SecretKey as TraitSecretKey, SignedMessage};
use base64::{encode, decode};
use std::fs::{File, OpenOptions};
use std::io::{Write, Read};
use std::path::Path;
use std::error::Error;
use crate::transaction::Transaction;

#[derive(Debug, Clone)]
pub struct Wallet {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
    pub balance: f64,
    pub kyc_verified: bool,
    pub agreed_to_terms: bool,
    pub recent_tx: Vec<Transaction>,
}

impl Wallet {
    pub fn generate() -> Self {
        let (pk, sk) = keypair();
        Wallet {
            public_key: pk,
            secret_key: sk,
            balance: 0.0,
            kyc_verified: false,
            agreed_to_terms: false,
            recent_tx: Vec::new(),
        }
    }

    pub fn get_address(&self) -> String {
        encode(self.public_key.as_bytes())
    }

    pub fn sign_detached(&self, msg: &[u8]) -> SignedMessage {
        sign_detached(msg, &self.secret_key)
    }

    pub fn verify_detached(&self, msg: &[u8], sig: &SignedMessage) -> bool {
        verify_detached(msg, sig, &self.public_key).is_ok()
    }

    pub fn get_balance(&self) -> f64 {
        self.balance
    }

    pub fn create_transaction(&mut self, recipient: String, amount: f64) -> Option<Transaction> {
        if amount <= 0.0 || amount > self.balance {
            return None;
        }

        self.balance -= amount;

        let tx = Transaction {
            sender: self.get_address(),
            recipient,
            amount,
            signature: Some(encode(self.sign_detached(format!("{recipient}{amount}").as_bytes()).as_bytes())),
        };

        self.recent_tx.push(tx.clone());

        if self.recent_tx.len() > 5 {
            self.recent_tx.remove(0);
        }

        Some(tx)
    }

    pub fn show_last_transactions(&self) {
        println!("\n📜 Last 5 Transactions:");
        for tx in &self.recent_tx {
            println!(
                "To: {} | Amount: {} QTC | Signature: {}",
                tx.recipient,
                tx.amount,
                tx.signature.as_deref().unwrap_or("None")
            );
        }
    }

    pub fn agree_to_terms(&mut self) {
        self.agreed_to_terms = true;
        println!("✅ Terms & Conditions accepted.");
    }

    pub fn verify_kyc(&mut self, input_code: &str) -> bool {
        // Placeholder logic; replace with real KYC service integration.
        if input_code == "KYC123456" {
            self.kyc_verified = true;
            println!("✅ KYC Verification Successful.");
            true
        } else {
            println!("❌ KYC Verification Failed.");
            false
        }
    }

    pub fn export_with_2fa(&self, password: &str) -> Result<(), Box<dyn Error>> {
        if password.len() < 8 {
            return Err("Password too short for 2FA export.".into());
        }

        let pub_key_encoded = encode(self.public_key.as_bytes());
        let sec_key_encoded = encode(self.secret_key.as_bytes());

        let mut file = File::create("wallet_backup.txt")?;
        writeln!(file, "PublicKey: {}", pub_key_encoded)?;
        writeln!(file, "SecretKey: {}", sec_key_encoded)?;
        writeln!(file, "2FA-Passcode: {}", password)?;
        writeln!(file, "AgreedToTerms: {}", self.agreed_to_terms)?;
        writeln!(file, "KYC Verified: {}", self.kyc_verified)?;
        Ok(())
    }

    pub fn save_to_files(&self) -> std::io::Result<()> {
        let pub_encoded = encode(self.public_key.as_bytes());
        let sec_encoded = encode(self.secret_key.as_bytes());

        let mut pub_file = File::create("wallet_pub.key")?;
        let mut sec_file = File::create("wallet_sec.key")?;

        pub_file.write_all(pub_encoded.as_bytes())?;
        sec_file.write_all(sec_encoded.as_bytes())?;

        Ok(())
    }

    pub fn load_from_files() -> Result<Self, Box<dyn Error>> {
        let mut pub_key_encoded = String::new();
        let mut sec_key_encoded = String::new();

        File::open("wallet_pub.key")?.read_to_string(&mut pub_key_encoded)?;
        File::open("wallet_sec.key")?.read_to_string(&mut sec_key_encoded)?;

        let pub_bytes = decode(pub_key_encoded)?;
        let sec_bytes = decode(sec_key_encoded)?;

        let public_key = PublicKey::from_bytes(&pub_bytes)?;
        let secret_key = SecretKey::from_bytes(&sec_bytes)?;

        Ok(Wallet {
            public_key,
            secret_key,
            balance: 0.0,
            kyc_verified: false,
            agreed_to_terms: false,
            recent_tx: Vec::new(),
        })
    }
}