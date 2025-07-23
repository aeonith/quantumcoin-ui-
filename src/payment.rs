use serde::Deserialize;
use reqwest;
use std::time::Duration;

const MINIMUM_USD: f64 = 10.0;

#[derive(Deserialize)]
struct PriceData {
    bitcoin: BitcoinPrice,
}

#[derive(Deserialize)]
struct BitcoinPrice {
    usd: f64,
}

pub async fn get_minimum_btc_amount() -> Result<f64, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd")
        .timeout(Duration::from_secs(5))
        .send()
        .await?;

    let data: PriceData = response.json().await?;
    let price_usd = data.bitcoin.usd;

    let btc_required = MINIMUM_USD / price_usd;
    Ok(btc_required)
}