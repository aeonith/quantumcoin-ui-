use std::time::Duration;
use reqwest::blocking::Client;
use serde::Deserialize;
use crate::wallet::{Wallet, load_wallet, send_tokens_to_wallet};
use crate::blockchain::{Blockchain, save_blockchain_to_file};

const BTC_ADDRESS: &str = "bc1qv7tpdxqvgwutfrhf53nhwgp77j5lv7whnk433y";

#[derive(Debug, Deserialize)]
struct PriceResponse {
    bitcoin: CoinPrice,
}

#[derive(Debug, Deserialize)]
struct CoinPrice {
    usd: f64,
}

pub fn fetch_bitcoin_price() -> Result<f64, Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd";

    let response = client.get(url).timeout(Duration::from_secs(10)).send()?;
    let price_data: PriceResponse = response.json()?;
    Ok(price_data.bitcoin.usd)
}

// Simulated BTC detection logic
pub fn check_and_dispatch_qtc(blockchain: &mut Blockchain) -> Result<(), Box<dyn std::error::Error>> {
    let btc_received = detect_btc_received()?; // mocked
    if btc_received > 0.0 {
        let btc_price = fetch_bitcoin_price()?;
        let usd_value = btc_received * btc_price;
        let qtc_to_send = (usd_value / 0.25).round() as u64;

        let mut user_wallet = load_wallet("wallet_key.json")?;
        send_tokens_to_wallet(blockchain, &mut user_wallet, qtc_to_send)?;
        save_blockchain_to_file(blockchain)?;
        println!("🎉 Dispatched {} QTC for {:.4} BTC", qtc_to_send, btc_received);
    }
    Ok(())
}

// MOCK: Replace this with actual BlockCypher/Mempool API webhook
fn detect_btc_received() -> Result<f64, Box<dyn std::error::Error>> {
    // Simulate 0.002 BTC received
    Ok(0.002)
}