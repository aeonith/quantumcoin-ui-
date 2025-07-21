use pqcrypto_dilithium::dilithium2::{keypair, sign_detached, verify_detached,
                                     PublicKey, SecretKey};
use pqcrypto_traits::sign::{DetachedSignature};
use serde::{Deserialize, Serialize};
use base64::{engine::general_purpose::STANDARD as b64, Engine};
use std::{fs, path::Path, error::Error};
use rand::Rng;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    /// Encoded public key (base64) – easy to (de)serialize.
    pub public_key_b64: String,
    #[serde(skip)]
    pub public_key: PublicKey,

    #[serde(skip)]
    pub secret_key: SecretKey,
}

impl Wallet {
    pub fn generate() -> Self {
        let (pk, sk) = keypair();
        Self {
            public_key_b64: b64.encode(pk.as_bytes()),
            public_key: pk,
            secret_key: sk,
        }
    }

    pub fn get_address(&self) -> String { self.public_key_b64.clone() }

    pub fn sign(&self, data: &[u8]) -> DetachedSignature {
        sign_detached(data, &self.secret_key)
    }

    pub fn verify(addr: &str, data: &[u8], sig: &DetachedSignature) -> bool {
        if let Ok(bytes) = b64.decode(addr) {
            if let Ok(pk) = PublicKey::from_bytes(&bytes) {
                return verify_detached(data, sig, &pk).is_ok();
            }
        }
        false
    }

    // ---------- Persistence ----------
    pub fn save_to_files(&self, dir: &str) -> Result<(), Box<dyn Error>> {
        fs::create_dir_all(dir)?;
        fs::write(format!("{dir}/wallet.json"),
                  serde_json::to_string_pretty(self)?)?;
        fs::write(format!("{dir}/wallet_sk.bin"), self.secret_key.as_bytes())?;
        Ok(())
    }

    pub fn load_from_files(dir: &str) -> Result<Self, Box<dyn Error>> {
        let txt = fs::read_to_string(format!("{dir}/wallet.json"))?;
        let mut w: Wallet = serde_json::from_str(&txt)?;
        // secret key lives in a separate binary file
        let sk_bytes = fs::read(format!("{dir}/wallet_sk.bin"))?;
        w.secret_key = SecretKey::from_bytes(&sk_bytes)?;
        // reconstruct the runtime PublicKey
        let pk_bytes = b64.decode(&w.public_key_b64)?;
        w.public_key = PublicKey::from_bytes(&pk_bytes)?;
        Ok(w)
    }

    // ---------- Helper calls used by CLI ----------
    pub fn get_balance(&self, bc: &crate::blockchain::Blockchain) -> u64 {
        bc.calculate_balance(&self.get_address())
    }

    pub fn create_transaction(&self, to: &str, amount: u64)
        -> crate::transaction::Transaction {
        crate::transaction::Transaction::new(&self.get_address(), to, amount)
    }

    pub fn export_with_2fa(&self) -> String {
        let code: u32 = rand::thread_rng().gen_range(100_000..=999_999);
        format!("{}:{}", self.public_key_b64, code)
    }

    pub fn show_last_transactions(&self,
                                  bc: &crate::blockchain::Blockchain,
                                  n: usize) {
        let mut count = 0;
        for block in bc.chain.iter().rev() {
            for tx in block.transactions.iter().rev() {
                if tx.from == self.get_address() || tx.to == self.get_address() {
                    println!("{tx:?}");
                    count += 1;
                    if count == n { return; }
                }
            }
        }
    }
}