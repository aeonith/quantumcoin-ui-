use std::time::{SystemTime, UNIX_EPOCH};
use crate::blockchain::Blockchain;
use crate::transaction::Transaction;

pub struct PriceEngine;

impl PriceEngine {
    pub fn calculate_price(blockchain: &Blockchain) -> f64 {
        let base_price = 0.25;
        let total_supply = 21_000_000.0;
        let circulating_supply = blockchain.total_circulating_supply();
        let recent_tx_count = Self::recent_transaction_count(blockchain);

        let supply_factor = circulating_supply / total_supply;
        let demand_factor = recent_tx_count as f64;

        let multiplier = if supply_factor == 0.0 {
            1.0
        } else {
            demand_factor / supply_factor
        };

        let price = base_price * multiplier;
        price.max(0.25)
    }

    fn recent_transaction_count(blockchain: &Blockchain) -> usize {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let one_hour_ago = now - 3600;

        blockchain
            .chain
            .iter()
            .flat_map(|block| &block.transactions)
            .filter(|tx| tx.timestamp >= one_hour_ago)
            .count()
    }
}