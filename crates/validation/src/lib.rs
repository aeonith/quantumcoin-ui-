use qc_crypto::{pq_verify, tx_sighash};
use qc_types::*;
use serde::Deserialize;
use thiserror::Error;
use pqcrypto_dilithium::dilithium2::PublicKey;

#[derive(Debug, Deserialize, Clone)]
pub struct ChainSpec {
    pub network: Network,
    pub consensus: Consensus,
    pub supply: Supply,
    pub txpolicy: TxPolicy,
    pub revstop: RevStop,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Network {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub version: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Consensus {
    pub hash_function: String,
    pub target_block_time_secs: u64,
    pub difficulty_adjustment: String,
    pub asert_half_life_secs: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Supply {
    pub max_supply_sats: i64,
    pub halving_interval_blocks: u64,
    pub premine_sats: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TxPolicy {
    pub max_tx_size: u64,
    pub min_fee_per_kb_sats: i64,
    pub dust_threshold_sats: i64,
    pub max_inputs: u32,
    pub max_outputs: u32,
    pub coinbase_maturity: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RevStop { 
    pub window_blocks: u32 
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("tx too large")] TxTooLarge,
    #[error("too many inputs/outputs")] CountLimit,
    #[error("dust output")] Dust,
    #[error("missing input")] MissingInput,
    #[error("insufficient funds")] InsufficientFunds,
    #[error("pq signature invalid")] BadSignature,
    #[error("revstop cancel outside window")] CancelOutsideWindow,
    #[error("revstop misuse")] RevstopMisuse,
    #[error("coinbase immature")] CoinbaseImmature,
}

fn encode_tx_skeleton(tx: &Transaction) -> Vec<u8> {
    let mut tmp = tx.clone();
    for i in &mut tmp.vin { 
        i.pq_signature.clear(); 
        i.cancel = false; 
    }
    bincode::serialize(&tmp).expect("serialize")
}

pub fn initial_subsidy_sats(spec: &ChainSpec, eras: u32) -> i64 {
    let blocks_per_era = spec.supply.halving_interval_blocks as i128;
    let cap = spec.supply.max_supply_sats as i128;
    let two_pow_eras = 1i128 << eras;
    let s0 = cap * two_pow_eras / (blocks_per_era * (two_pow_eras - 1));
    s0 as i64
}

pub fn block_subsidy(spec: &ChainSpec, height: u64) -> i64 {
    let era = height / spec.supply.halving_interval_blocks;
    let s0 = initial_subsidy_sats(spec, 64);
    let sub = s0 >> era.min(63) as u32;
    if sub < 0 { 0 } else { sub }
}

pub fn validate_transaction<FLookup>(
    spec: &ChainSpec,
    height_now: u64,
    tx: &Transaction,
    is_coinbase: bool,
    mut lookup: FLookup
) -> Result<(), ValidationError>
where
    FLookup: FnMut(&OutPoint) -> Option<(Amount, OutputType, Height, bool)>
{
    // size & shape
    let sz = bincode::serialize(tx).map(|v| v.len()).unwrap_or(usize::MAX);
    if sz as u64 > spec.txpolicy.max_tx_size { return Err(ValidationError::TxTooLarge); }
    if tx.vin.len() as u32 > spec.txpolicy.max_inputs || tx.vout.len() as u32 > spec.txpolicy.max_outputs {
        return Err(ValidationError::CountLimit);
    }
    for o in &tx.vout {
        if o.value < spec.txpolicy.dust_threshold_sats { return Err(ValidationError::Dust); }
    }

    if is_coinbase { return Ok(()); }

    let mut sum_in: i128 = 0;
    let sum_out: i128 = tx.vout.iter().map(|o| o.value as i128).sum();

    let skeleton = encode_tx_skeleton(tx);
    let sighash = tx_sighash(&skeleton);

    for input in &tx.vin {
        let Some((val, out_type, created_height, was_coinbase)) = lookup(&input.prevout) else {
            return Err(ValidationError::MissingInput);
        };

        if was_coinbase {
            if height_now.saturating_sub(created_height) < spec.txpolicy.coinbase_maturity as u64 {
                return Err(ValidationError::CoinbaseImmature);
            }
        }

        match &out_type {
            OutputType::P2PQ { pubkey } => {
                if input.cancel { return Err(ValidationError::RevstopMisuse); }
                if !pq_verify_pub(pubkey, &sighash, &input.pq_signature) {
                    return Err(ValidationError::BadSignature);
                }
            }
            OutputType::P2PQRevocable { pubkey, window_blocks } => {
                let age = height_now.saturating_sub(created_height);
                if input.cancel {
                    if age > *window_blocks as u64 { return Err(ValidationError::CancelOutsideWindow); }
                    if !pq_verify_pub(pubkey, &sighash, &input.pq_signature) {
                        return Err(ValidationError::BadSignature);
                    }
                } else {
                    if !pq_verify_pub(pubkey, &sighash, &input.pq_signature) {
                        return Err(ValidationError::BadSignature);
                    }
                }
            }
        }

        sum_in += val as i128;
    }

    if sum_in < sum_out { return Err(ValidationError::InsufficientFunds); }
    Ok(())
}

fn pq_verify_pub(pubkey: &Vec<u8>, sighash: &[u8;32], sig: &Vec<u8>) -> bool {
    match PublicKey::from_bytes(pubkey.clone()) {
        Ok(pk) => pq_verify(&pk, sighash, sig),
        Err(_) => false,
    }
}

pub fn merkle_root(txs: &[Transaction]) -> Hash32 {
    use sha2::{Digest, Sha256};
    fn h(bytes: &[u8]) -> [u8;32] {
        let mut sh = Sha256::new();
        sh.update(bytes);
        let out = sh.finalize();
        let mut arr = [0u8;32]; arr.copy_from_slice(&out); arr
    }
    let mut layer: Vec<[u8;32]> = txs.iter().map(|t| h(&bincode::serialize(t).unwrap())).collect();
    if layer.is_empty() { return Hash32::zero(); }
    while layer.len() > 1 {
        let mut next = vec![];
        for i in (0..layer.len()).step_by(2) {
            let a = layer[i];
            let b = if i+1 < layer.len() { layer[i+1] } else { layer[i] };
            let mut sh = Sha256::new();
            sh.update(a); sh.update(b);
            let out = sh.finalize();
            let mut arr = [0u8;32]; arr.copy_from_slice(&out);
            next.push(arr);
        }
        layer = next;
    }
    Hash32(layer[0])
}
