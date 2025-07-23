use reqwest::get;
use serde::Deserialize;

#[derive(Deserialize)]
struct BtcResponse {
    bitcoin: BtcPrice,
}

#[derive(Deserialize)]
struct BtcPrice {
    usd: f64,
}

pub async fn get_btc_price_usd() -> Option<f64> {
    let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd";
    match get(url).await {
        Ok(resp) => {
            let data: BtcResponse = resp.json().await.ok()?;
            Some(data.bitcoin.usd)
        }
        Err(_) => None,
    }
}