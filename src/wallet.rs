use pqcrypto_dilithium::dilithium2::{keypair, PublicKey, SecretKey, sign_detached};
use pqcrypto_traits::sign::{PublicKey as TraitPublicKey, SecretKey as TraitSecretKey};
use base64::{encode, decode};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use crate::transaction::Transaction;

pub struct Wallet {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
}

impl Wallet {
    pub fn new() -> Self {
        let (pk, sk) = keypair();
        Wallet {
            public_key: pk,
            secret_key: sk,
        }
    }

    pub fn get_address(&self) -> String {
        encode(self.public_key.as_bytes())
    }

    pub fn get_balance(&self, blockchain: &crate::blockchain::Blockchain) -> f64 {
        blockchain.get_balance(&self.get_address())
    }

    pub fn create_transaction(&self, to: &str, amount: f64) -> Transaction {
        let tx = Transaction {
            sender: self.get_address(),
            recipient: to.to_string(),
            amount,
            signature: None,
        };

        let tx_data = format!("{}{}{}", tx.sender, tx.recipient, tx.amount);
        let signature = sign_detached(tx_data.as_bytes(), &self.secret_key);
        Transaction {
            signature: Some(encode(signature.as_bytes())),
            ..tx
        }
    }

    pub fn save_to_files(&self) {
        let pub_str = encode(self.public_key.as_bytes());
        let priv_str = encode(self.secret_key.as_bytes());

        let mut pub_file = File::create("wallet_public.key").unwrap();
        pub_file.write_all(pub_str.as_bytes()).unwrap();

        let mut priv_file = File::create("wallet_private.key").unwrap();
        priv_file.write_all(priv_str.as_bytes()).unwrap();
    }

    pub fn load_from_files() -> Option<Self> {
        if !Path::new("wallet_public.key").exists() || !Path::new("wallet_private.key").exists() {
            return None;
        }

        let mut pub_contents = String::new();
        File::open("wallet_public.key").ok()?.read_to_string(&mut pub_contents).ok()?;

        let mut priv_contents = String::new();
        File::open("wallet_private.key").ok()?.read_to_string(&mut priv_contents).ok()?;

        let pub_bytes = decode(pub_contents).ok()?;
        let priv_bytes = decode(priv_contents).ok()?;

        let public_key = PublicKey::from_bytes(&pub_bytes).ok()?;
        let secret_key = SecretKey::from_bytes(&priv_bytes).ok()?;

        Some(Wallet { public_key, secret_key })
    }

    pub fn show_wallet_address(&self) {
        println!("🔐 Your Wallet Address:\n{}", self.get_address());
    }

    pub fn export_with_2fa(&self) {
        use rand::{distributions::Alphanumeric, Rng};
        use std::time::Duration;
        use std::thread;

        let code: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();

        println!("📲 2FA Code Sent: {}", code);
        println!("Please enter the 2FA code to continue:");
        thread::sleep(Duration::from_secs(1));

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        if input.trim() == code {
            println!("✅ 2FA verified. Exporting wallet...");
            self.save_to_files();
            println!("🗂️ Wallet exported as 'wallet_public.key' and 'wallet_private.key'");
        } else {
            println!("❌ Incorrect 2FA code. Export canceled.");
        }
    }
}