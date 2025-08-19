use anyhow::*;
use parking_lot::Mutex;
use rand::{Rng, thread_rng};
use serde::{Serialize, Deserialize};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

pub type Hash = [u8;32];

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Tx {
    pub nonce: u64,
    pub from: String,
    pub to: String,
    pub value: u64,
    pub fee: u64,
    pub data: String
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BlockHeader {
    pub parent: String,
    pub number: u64,
    pub timestamp: u64,
    pub difficulty: u128,
    pub nonce: u64,
    pub merkle_root: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Block {
    pub hash: String,
    pub header: BlockHeader,
    pub txs: Vec<Tx>,
    pub work: u128, // difficulty contribution
}

#[derive(Default)]
struct ChainInner {
    blocks_by_hash: HashMap<String, Block>,
    hash_by_number: HashMap<u64, String>,
    head: String,
    total_work: u128,
    peers: u64,
}

#[derive(Clone)]
pub struct Chain(Arc<Mutex<ChainInner>>);

impl Chain {
    pub fn new_genesis() -> Self {
        let inner = ChainInner::default();
        let me = Self(Arc::new(Mutex::new(inner)));
        let genesis = Self::make_block(None, 0, 0x0000_0fff_ffff_ffff_ffff, vec![]);
        let mut g = me.0.lock();
        g.total_work = genesis.work;
        g.hash_by_number.insert(0, genesis.hash.clone());
        g.blocks_by_hash.insert(genesis.hash.clone(), genesis.clone());
        g.head = genesis.hash.clone();
        g.peers = 1;
        me
    }

    fn make_block(parent: Option<&Block>, number: u64, difficulty: u128, txs: Vec<Tx>) -> Block {
        let parent_hash = parent.map(|b| b.hash.clone()).unwrap_or_else(|| "0x00".into());
        let merkle_root = merkle_root(&txs);
        let timestamp = now();
        let mut nonce = 0u64;
        // naive PoW: find nonce s.t. hash_u128 <= target
        let mut rng = thread_rng();
        let target = u128::MAX / difficulty;
        let header_seed = |nonce: u64| {
            let mut h = Sha256::new();
            h.update(&hex::decode(parent_hash.trim_start_matches("0x")).unwrap_or_default());
            h.update(number.to_be_bytes());
            h.update(timestamp.to_be_bytes());
            h.update(difficulty.to_be_bytes());
            h.update(nonce.to_be_bytes());
            h.update(&hex::decode(merkle_root.trim_start_matches("0x")).unwrap_or_default());
            let first = h.finalize();
            let mut h2 = Sha256::new();
            h2.update(first);
            let out = h2.finalize();
            let mut arr=[0u8;32]; arr.copy_from_slice(&out); arr
        };
        let hash_u128 = |bytes: &Hash| -> u128 {
            let mut n = [0u8;16];
            n.copy_from_slice(&bytes[..16]);
            u128::from_be_bytes(n)
        };
        let mut hash_bytes = header_seed(nonce);
        while hash_u128(&hash_bytes) > target {
            nonce = nonce.wrapping_add(1).max(rng.gen::<u32>() as u64);
            hash_bytes = header_seed(nonce);
        }
        let hash = format!("0x{}", hex::encode(hash_bytes));
        let header = BlockHeader { parent: parent_hash, number, timestamp, difficulty, nonce, merkle_root };
        let work = difficulty;
        Block { hash, header, txs, work }
    }

    pub fn head(&self) -> Block { self.0.lock().blocks_by_hash[&self.0.lock().head].clone() }
    pub fn height(&self) -> u64 { self.0.lock().hash_by_number.len().saturating_sub(1) as u64 }
    pub fn peers(&self) -> u64 { self.0.lock().peers }

    pub fn get_block_by_number(&self, n: u64) -> Option<Block> {
        let g = self.0.lock();
        g.hash_by_number.get(&n).and_then(|h| g.blocks_by_hash.get(h).cloned())
    }

    pub fn mine_one(&self) -> Block {
        // simplistic retarget: keep target ~30s by adjusting difficulty ±5%
        let mut g = self.0.lock();
        let prev = g.blocks_by_hash.get(&g.head).unwrap();
        let last_ts = prev.header.timestamp;
        let target = 30u64;
        let mut difficulty = prev.header.difficulty;
        let dt = now().saturating_sub(last_ts).max(1);
        if dt < target { difficulty = (difficulty as f64 * 1.05) as u128; }
        if dt > target { difficulty = (difficulty as f64 * 0.95) as u128; }
        difficulty = difficulty.clamp(1_000_000, u128::MAX/2);

        let b = Self::make_block(Some(prev), prev.header.number+1, difficulty, vec![]);
        g.blocks_by_hash.insert(b.hash.clone(), b.clone());
        g.hash_by_number.insert(b.header.number, b.hash.clone());
        g.head = b.hash.clone();
        g.total_work += b.work;
        b
    }
}

fn merkle_root(txs:&[Tx])->String{
    if txs.is_empty(){ return format!("0x{}", hex::encode([0u8;32])); }
    let mut hashes: Vec<Hash> = txs.iter().map(|t|{
        let mut h=Sha256::new(); h.update(serde_json::to_vec(t).unwrap()); let first=h.finalize();
        let mut h2=Sha256::new(); h2.update(first); let out=h2.finalize();
        let mut a=[0u8;32]; a.copy_from_slice(&out); a
    }).collect();
    while hashes.len()>1{
        let mut next=Vec::new();
        for pair in hashes.chunks(2){
            let a=pair[0]; let b=*pair.get(1).unwrap_or(&pair[0]);
            let mut h=Sha256::new(); h.update(a); h.update(b);
            let first=h.finalize(); let mut h2=Sha256::new(); h2.update(first);
            let out=h2.finalize(); let mut arr=[0u8;32]; arr.copy_from_slice(&out); next.push(arr);
        }
        hashes=next;
    }
    format!("0x{}", hex::encode(hashes[0]))
}

fn now()->u64{
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}
