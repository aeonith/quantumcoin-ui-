use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetworkMessage {
    NewTransaction(String), // Serialized JSON of your transaction
    NewBlock(String),       // Serialized JSON of your block
    RequestChain,           // Ask for latest blocks
    ChainResponse(Vec<String>), // Serialized block chain
}