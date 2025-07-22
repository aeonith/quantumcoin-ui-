use crate::blockchain::{Blockchain, Transaction, Block};
use crate::revstop::RevStop;
use pqcrypto_dilithium::dilithium3::PublicKey;
use rand::Rng;
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};

const DIFFICULTY_PREFIX: &str = "0000";
const MINING_REWARD: f64 = 50.0;

pub fn mine_pending_transactions(
    blockchain: &mut Blockchain,
    miner_address: &str,
    revstop: &RevStop,
) {
    if revstop.is_enabled() {
        println!("🚫 Mining is currently locked by RevStop.");
        return;
    }

    let pending = blockchain.take_pending();
    if pending.is_empty() {
        println!("📭 No pending transactions to mine.");
        return;
    }

    println!("⚒️ Mining in progress... (difficulty: {})", DIFFICULTY_PREFIX);

    let reward_tx = Transaction {
        sender: "network".to_string(),
        recipient: miner_address.to_string(),
        amount: MINING_REWARD,
        signature: None,
    };

    let mut block = Block::new(blockchain.get_last_hash(), [vec![reward_tx], pending].concat());

    loop {
        block.nonce = rand::thread_rng().gen();
        let hash = block.calculate_hash();
        if hash.starts_with(DIFFICULTY_PREFIX) {
            block.hash = hash;
            break;
        }
    }

    println!("✅ Block successfully mined with hash: {}", block.hash);
    blockchain.add_block(block);
    blockchain.save_to_disk();
}