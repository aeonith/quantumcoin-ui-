use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount: f64,
    pub signature: Option<String>,
}

impl Transaction {
    pub fn new(sender: String, recipient: String, amount: f64) -> Self {
        Transaction {
            sender,
            recipient,
            amount,
            signature: None,
        }
    }

    pub fn set_signature(&mut self, signature: Vec<u8>) {
        self.signature = Some(base64::encode(&signature));
    }

    pub fn get_signature_bytes(&self) -> Option<Vec<u8>> {
        self.signature
            .as_ref()
            .and_then(|sig| base64::decode(sig).ok())
    }
}