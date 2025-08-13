use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

const BASE_PRICE: f64 = 0.25;
const PRICE_HISTORY_FILE: &str = "data/price_history.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PriceSnapshot {
    pub timestamp: u64,
    pub price: f64,
}

pub fn calculate_live_price(
    available_supply: u64,
    active_buy_requests: u64,
    active_sell_requests: u64,
) -> f64 {
    let supply = available_supply.max(1) as f64;
    let demand_index = active_buy_requests as f64 / supply;
    let supply_index = active_sell_requests as f64 / supply;

    let multiplier = (demand_index / (supply_index + 1.0)).clamp(0.5, 3.0);
    let calculated_price = (BASE_PRICE * multiplier).max(BASE_PRICE);

    save_price_snapshot(calculated_price);
    calculated_price
}

fn save_price_snapshot(price: f64) {
    let snapshot = PriceSnapshot {
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        price,
    };

    let mut history = load_price_history();
    history.push(snapshot);

    if let Ok(json) = serde_json::to_string_pretty(&history) {
        let _ = fs::create_dir_all("data");
        let _ = fs::write(PRICE_HISTORY_FILE, json);
    }
}

fn load_price_history() -> Vec<PriceSnapshot> {
    if Path::new(PRICE_HISTORY_FILE).exists() {
        let mut file = File::open(PRICE_HISTORY_FILE).unwrap_or_else(|_| File::create(PRICE_HISTORY_FILE).unwrap());
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap_or(0);

        if let Ok(history) = serde_json::from_str(&contents) {
            return history;
        }
    }
    Vec::new()
}