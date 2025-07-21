use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount: f64,
    pub signature: Option<String>,
}

impl Transaction {
    pub fn is_valid(&self) -> bool {
        if self.sender == "GENESIS" {
            return true;
        }

        if self.signature.is_none() {
            return false;
        }

        let data = format!("{}{}{}", self.sender, self.recipient, self.amount);
        let signature = match base64::decode(self.signature.as_ref().unwrap()) {
            Ok(sig) => sig,
            Err(_) => return false,
        };

        let pub_key_bytes = match base64::decode(&self.sender) {
            Ok(bytes) => bytes,
            Err(_) => return false,
        };

        let pub_key = match pqcrypto_dilithium::dilithium2::PublicKey::from_bytes(&pub_key_bytes) {
            Ok(pk) => pk,
            Err(_) => return false,
        };

        pqcrypto_dilithium::dilithium2::verify_detached(&signature, data.as_bytes(), &pub_key).is_ok()
    }
}