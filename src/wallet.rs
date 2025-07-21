use crate::{blockchain::Blockchain, transaction::Transaction};
use pqcrypto_dilithium::dilithium2::{keypair, PublicKey, SecretKey};
use serde::{Deserialize, Serialize};
use std::{error::Error, fs};

#[derive(Serialize, Deserialize)]
pub struct Wallet {
    pub address: String,
    #[serde(skip)]
    pub public_key: PublicKey,
    #[serde(skip)]
    pub secret_key: SecretKey,
}

impl std::fmt::Debug for Wallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Wallet")
            .field("address", &self.address)
            .finish() // keep keys out of debug output
    }
}

impl Wallet {
    pub fn generate() -> Self {
        let (pk, sk) = keypair();
        let address = base64::encode(pk.as_bytes());
        Self { address, public_key: pk, secret_key: sk }
    }

    pub fn save_to_files(&self) -> Result<(), Box<dyn Error>> {
        fs::write("wallet.json", serde_json::to_vec_pretty(self)?)?;
        fs::write("wallet_sk.bin", self.secret_key.as_bytes())?;
        Ok(())
    }

    pub fn load_from_files() -> Result<Self, Box<dyn Error>> {
        let bytes = fs::read("wallet.json")?;
        let mut wallet: Wallet = serde_json::from_slice(&bytes)?;
        let sk_bytes = fs::read("wallet_sk.bin")?;
        wallet.secret_key = SecretKey::from_bytes(&sk_bytes)?;
        wallet.public_key = wallet.secret_key.to_public_key();
        Ok(wallet)
    }

    pub fn get_address(&self) -> &str {
        &self.address
    }

    pub fn get_balance(&self, bc: &Blockchain) -> f64 {
        let mut bal = 0.0;
        for block in &bc.chain {
            for tx in &block.transactions {
                if tx.recipient == self.address {
                    bal += tx.amount;
                } else if tx.sender == self.address {
                    bal -= tx.amount;
                }
            }
        }
        bal
    }

    pub fn create_transaction(&self, recipient: &str, amount: f64) -> Transaction {
        Transaction::new(&self.address, recipient, amount)
    }

    pub fn show_last_transactions(&self, bc: &Blockchain) {
        for block in bc.chain.iter().rev().take(5) {
            for tx in &block.transactions {
                if tx.sender == self.address || tx.recipient == self.address {
                    println!("{tx:?}");
                }
            }
        }
    }

    /* ----- simple / fake stubs you can extend later ----- */

    pub fn export_with_2fa(&self) {
        println!("🔑 (stub) wallet exported – implement real 2FA here");
    }
}