use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct PriceData {
    bitcoin: Currency,
}

#[derive(Deserialize)]
struct Currency {
    usd: f64,
}

/// Fetches the current Bitcoin price in USD from CoinGecko.
pub async fn get_btc_price_usd() -> Option<f64> {
    let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd";
    let client = Client::new();

    match client.get(url).send().await {
        Ok(resp) => {
            if let Ok(data) = resp.json::<PriceData>().await {
                Some(data.bitcoin.usd)
            } else {
                None
            }
        }
        Err(_) => None,
    }
}