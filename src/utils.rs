use reqwest;
use serde_json::Value;

pub async fn get_btc_price_usd() -> Result<f64, reqwest::Error> {
    let url = "https://api.coindesk.com/v1/bpi/currentprice/BTC.json";
    let response = reqwest::get(url).await?;
    let data: Value = response.json().await?;
    let price = data["bpi"]["USD"]["rate_float"].as_f64().unwrap_or(0.0);
    Ok(price)
}