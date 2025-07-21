use crate::blockchain::Blockchain;
use pqcrypto_dilithium::dilithium2::{keypair, PublicKey, SecretKey, sign_detached, verify_detached};
use base64::{engine::general_purpose::STANDARD as b64, Engine as _};
use serde::{Deserialize, Serialize};
use std::{fs, error::Error, path::Path};
use rand::Rng;

#[derive(Serialize, Deserialize, Debug)]
pub struct Wallet {
    pub address: String,             // base64 of public key
    #[serde(skip)] pub pk: PublicKey,
    #[serde(skip)] pub sk: SecretKey,
}

impl Wallet {
    pub fn generate_and_save(dir: &str) -> Result<Self, Box<dyn Error>> {
        let (pk, sk) = keypair();
        let addr = b64.encode(pk.as_bytes());
        let w = Wallet { address: addr, pk, sk };
        w.save(dir)?;
        Ok(w)
    }

    pub fn load_from_files(dir: &str) -> Result<Option<Self>, Box<dyn Error>> {
        if !Path::new(dir).exists() { return Ok(None); }
        let meta = fs::read_to_string(format!("{}/wallet.json", dir))?;
        let mut w: Wallet = serde_json::from_str(&meta)?;
        let skb = fs::read(format!("{}/wallet_sk.bin", dir))?;
        w.sk = SecretKey::from_bytes(&skb)?;
        let pkb = b64.decode(&w.address)?;
        w.pk = PublicKey::from_bytes(&pkb)?;
        Ok(Some(w))
    }

    pub fn save(&self, dir: &str) -> Result<(), Box<dyn Error>> {
        if !Path::new(dir).exists() { fs::create_dir_all(dir)?; }
        fs::write(
            format!("{}/wallet.json", dir),
            serde_json::to_string_pretty(&self)?,
        )?;
        fs::write(
            format!("{}/wallet_sk.bin", dir),
            self.sk.as_bytes(),
        )?;
        println!("✅ Wallet saved to `{}`", dir);
        Ok(())
    }

    pub fn get_address(&self) -> String { self.address.clone() }

    pub fn get_balance(&self, bc: &Blockchain) -> u64 {
        bc.get_balance(&self.address)
    }

    pub fn create_transaction(&self, to: &str, amount: u64) -> crate::transaction::Transaction {
        crate::transaction::Transaction::new(&self.address, to, amount)
    }

    pub fn export_with_2fa(&self) {
        let code: u32 = rand::thread_rng().gen_range(100_000..1_000_000);
        println!("2FA code: {}", code);
        // In production, email this code...
    }
}