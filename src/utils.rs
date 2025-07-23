use reqwest;
use serde_json::Value;

pub async fn get_btc_price_usd() -> Result<f64, reqwest::Error> {
    let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd";
    let response = reqwest::get(url).await?;
    let json: Value = response.json().await?;
    Ok(json["bitcoin"]["usd"].as_f64().unwrap_or(0.0))
}